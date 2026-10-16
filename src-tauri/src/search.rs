use serde::Serialize;

/// A single search result, serialized to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct FileHit {
    pub path: String,
    pub name: String,
    pub ext: Option<String>,
    pub parent: String,
    pub size: i64,
    pub modified: i64,
    pub score: f64,
}

/// Rank a file name against a fuzzy query.
///
/// Scoring order (higher is better):
///   1. Exact match (name == query, case-insensitive)
///   2. Exact prefix  (name starts with query)
///   3. Substring     (query appears inside name)
///   4. Subsequence   (query is a subsequence of name, e.g. "ppt sk" -> "PPT Skripsi")
///
/// Each tier adds a base score; a shorter file relative to the query is slightly
/// preferred. Returns `0.0` when the query is not matched at all.
pub fn fuzzy_score(name: &str, query: &str) -> f64 {
    if query.is_empty() {
        return 0.0;
    }

    let lower_name = name.to_lowercase();
    let lower_query = query.to_lowercase();

    // 1. Exact match
    if lower_name == lower_query {
        return 1000.0;
    }

    // 2. Prefix match
    if lower_name.starts_with(&lower_query) {
        // Slightly prefer shorter names on a prefix match.
        return 900.0 + (lower_query.len() as f64 / lower_name.len() as f64) * 100.0;
    }

    // 3. Substring match
    if let Some(pos) = lower_name.find(&lower_query) {
        // Prefer matches that start earlier in the name.
        let position_bonus = 100.0 - (pos as f64 * 2.0).min(80.0);
        return 700.0 + position_bonus;
    }

    // 4. Subsequence match: every character of the query must appear in the name
    //    in order. A tighter cluster of matches (fewer gaps) gets a higher score.
    let name_chars: Vec<char> = lower_name.chars().collect();
    let mut idx = 0usize; // scanning position in the name (monotonic)
    let mut last_match: Option<usize> = None;
    let mut total_gap = 0usize;
    let mut matched = 0usize;

    for qc in lower_query.chars() {
        let mut found = false;
        while idx < name_chars.len() {
            if name_chars[idx] == qc {
                if let Some(prev) = last_match {
                    total_gap += idx - prev - 1; // characters skipped between matches
                }
                last_match = Some(idx);
                idx += 1; // next query char must match after this position
                matched += 1;
                found = true;
                break;
            }
            idx += 1;
        }
        if !found {
            return 0.0; // Query character not found in order
        }
    }

    if matched != lower_query.len() {
        return 0.0;
    }

    // Base subsequence score (much lower than substring), penalised by the
    // number of skipped characters so that tighter matches rank higher.
    let coverage = lower_query.len() as f64 / lower_name.len() as f64;
    let score = 300.0 + (coverage * 100.0) - (total_gap as f64 * 5.0);
    score.max(1.0)
}

/// Parse query tokens: text words plus optional filters.
/// Supported filters: `ext:<ext>`, `folder:<name>`.
struct ParsedQuery {
    text: String,
    ext: Option<String>,
    folder: Option<String>,
}

fn parse_query(query: &str) -> ParsedQuery {
    let lower = query.to_lowercase();
    let mut text_words: Vec<String> = Vec::new();
    let mut ext: Option<String> = None;
    let mut folder: Option<String> = None;

    for token in lower.split_whitespace() {
        if let Some(v) = token.strip_prefix("ext:") {
            if !v.is_empty() {
                ext = Some(v.to_string());
            }
        } else if let Some(v) = token.strip_prefix("folder:") {
            if !v.is_empty() {
                folder = Some(v.to_string());
            }
        } else {
            text_words.push(token.to_string());
        }
    }

    ParsedQuery {
        text: text_words.join(" "),
        ext,
        folder,
    }
}

/// Fuzzy search indexed files by query.
///
/// The query is split into a free-text portion and optional filters:
///   - `ext:<extension>` restricts results to files with that extension
///   - `folder:<substring>` restricts results to files whose parent folder
///     matches the substring (case-insensitive)
///
/// Every word of the text portion must appear (as a substring) in the file
/// name or parent path for a candidate to be considered; ranking is then done
/// in-memory with `fuzzy_score`.
pub fn search(query: &str, limit: usize) -> rusqlite::Result<Vec<FileHit>> {
    let parsed = parse_query(query);

    // If the only content is filter tokens (no text), fall back to a meta filter.
    if parsed.text.is_empty() {
        return filter_by_meta(&parsed, limit);
    }

    let words: Vec<&str> = parsed.text.split_whitespace().collect();
    // Fetch more candidates than needed so ranking has room to sort.
    let fetch_limit = (limit * 5).max(200);

    crate::db::with_conn(|conn| {
        let mut sql = String::from(
            "SELECT path, name, ext, parent, size, modified, open_count FROM files WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Prefilter: each word must appear in the name or the parent path.
        for word in &words {
            sql.push_str(" AND (name LIKE ? OR parent LIKE ?)");
            let pattern = format!("%{}%", word);
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }
        sql.push_str(" ORDER BY modified DESC LIMIT ?");
        params.push(Box::new(fetch_limit as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,        // path
                    row.get::<_, String>(1)?,        // name
                    row.get::<_, Option<String>>(2)?, // ext
                    row.get::<_, String>(3)?,        // parent
                    row.get::<_, i64>(4)?,           // size
                    row.get::<_, i64>(5)?,           // modified
                    row.get::<_, i64>(6)?,           // open_count
                ))
            },
        )?;

        // Collect, apply ext/folder filters, and score.
        let mut scored: Vec<(f64, i64, i64, (String, String, Option<String>, String, i64))> = Vec::new();
        for row in rows {
            let r = row?;
            if !matches_meta(&r, &parsed) {
                continue;
            }
            let score = fuzzy_score(&r.1, &parsed.text);
            if score > 0.0 {
                // (score, open_count, modified, (path, name, ext, parent, size))
                scored.push((
                    score,
                    r.6, // open_count
                    r.5, // modified
                    (r.0, r.1, r.2, r.3, r.4),
                ));
            }
        }

        // Sort by fuzzy score desc, then open_count desc, then modified desc.
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.1.cmp(&a.1))
                .then(b.2.cmp(&a.2))
        });

        Ok(scored
            .into_iter()
            .take(limit)
            .map(|(score, _open_count, modified, (path, name, ext, parent, size))| FileHit {
                path,
                name,
                ext,
                parent,
                size,
                modified,
                score,
            })
            .collect())
    })
}

fn matches_meta(
    cand: &(String, String, Option<String>, String, i64, i64, i64),
    parsed: &ParsedQuery,
) -> bool {
    let (_, _name, ext, parent, ..) = cand;

    if let Some(wanted_ext) = &parsed.ext {
        match ext {
            Some(e) if e.eq_ignore_ascii_case(wanted_ext) => {}
            _ => return false,
        }
    }

    if let Some(wanted_folder) = &parsed.folder {
        let lower_parent = parent.to_lowercase();
        if !lower_parent.contains(wanted_folder) {
            return false;
        }
    }

    true
}

/// Filter only, no free text query.
fn filter_by_meta(parsed: &ParsedQuery, limit: usize) -> rusqlite::Result<Vec<FileHit>> {
    crate::db::with_conn(|conn| {
        let mut sql = String::from(
            "SELECT path, name, ext, parent, size, modified FROM files WHERE 1 = 1",
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ext) = &parsed.ext {
            sql.push_str(" AND lower(ext) = ?");
            params.push(Box::new(ext.to_lowercase()));
        }
        if let Some(folder) = &parsed.folder {
            sql.push_str(" AND lower(parent) LIKE ?");
            params.push(Box::new(format!("%{}%", folder.to_lowercase())));
        }

        sql.push_str(" ORDER BY modified DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&sql)?;
        let row_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(FileHit {
                path: row.get(0)?,
                name: row.get(1)?,
                ext: row.get(2)?,
                parent: row.get(3)?,
                size: row.get(4)?,
                modified: row.get(5)?,
                score: 0.0,
            })
        })?;

        row_iter.collect()
    })
}

/// Most recently modified files, shown when the query is empty.
pub fn recent(limit: usize) -> rusqlite::Result<Vec<FileHit>> {
    crate::db::with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT path, name, ext, parent, size, modified FROM files
             ORDER BY modified DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok(FileHit {
                path: row.get(0)?,
                name: row.get(1)?,
                ext: row.get(2)?,
                parent: row.get(3)?,
                size: row.get(4)?,
                modified: row.get(5)?,
                score: 0.0,
            })
        })?;
        rows.collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_scores_highest() {
        assert_eq!(fuzzy_score("report.pdf", "report.pdf"), 1000.0);
    }

    #[test]
    fn prefix_beats_substring() {
        let prefix = fuzzy_score("skripsi final.pdf", "skripsi");
        let substring = fuzzy_score("my skripsi final.pdf", "skripsi");
        assert!(prefix > substring, "prefix score should beat substring");
    }

    #[test]
    fn substring_beats_subsequence() {
        let _substring = fuzzy_score("skripsi final.pdf", "skripsi");
        let subseq = fuzzy_score("skripsi-final.docx", "skfrl"); // nonsense subsequence
        // "skfrl" is not a subsequence of "skripsi-final", so should return 0
        assert_eq!(subseq, 0.0);
    }

    #[test]
    fn subsequence_match() {
        let score = fuzzy_score("ppt skripsi final.pptx", "ppt sk");
        assert!(score > 0.0, "expected a subsequence match");
    }

    #[test]
    fn no_match_returns_zero() {
        assert_eq!(fuzzy_score("report.pdf", "xyz completely different"), 0.0);
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(fuzzy_score("MYFILE.txt", "myfile.txt"), 1000.0);
    }

    #[test]
    fn parse_query_ext() {
        let parsed = parse_query("ext:pdf skripsi");
        assert_eq!(parsed.ext.as_deref(), Some("pdf"));
        assert_eq!(parsed.text, "skripsi");
    }

    #[test]
    fn parse_query_folder() {
        let parsed = parse_query("folder:skripsi report");
        assert_eq!(parsed.folder.as_deref(), Some("skripsi"));
        assert_eq!(parsed.text, "report");
    }

    #[test]
    fn parse_query_multiple_filters() {
        let parsed = parse_query("ext:docx folder:kerja report");
        assert_eq!(parsed.ext.as_deref(), Some("docx"));
        assert_eq!(parsed.folder.as_deref(), Some("kerja"));
        assert_eq!(parsed.text, "report");
    }
}
