use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

/// Global database handle, initialized during app setup.
pub static DB: Mutex<Option<Connection>> = Mutex::new(None);

/// A file entry to insert/update in the index.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub ext: Option<String>,
    pub parent: String,
    pub size: i64,
    pub modified: i64,
}

/// Location of the SQLite index file: %APPDATA%/cariyuk/index.db
pub fn db_path() -> PathBuf {
    directories::ProjectDirs::from("com", "cariyuk", "app")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("index.db")
}

/// Open (or create) the index database and set up the schema.
pub fn init() -> rusqlite::Result<()> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS files (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            path       TEXT UNIQUE NOT NULL,
            name       TEXT NOT NULL,
            ext        TEXT,
            parent     TEXT NOT NULL,
            size       INTEGER NOT NULL,
            modified   INTEGER NOT NULL,
            open_count INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_files_name ON files(name);
        CREATE INDEX IF NOT EXISTS idx_files_ext ON files(ext);
        CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified DESC);
        ",
    )?;
    *DB.lock().unwrap() = Some(conn);
    Ok(())
}

/// Run a closure with the shared connection.
pub fn with_conn<T>(f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> rusqlite::Result<T> {
    let guard = DB.lock().unwrap();
    let conn = guard.as_ref().expect("database not initialized");
    f(conn)
}

/// Insert or update a batch of file entries in a single transaction.
pub fn upsert_many(entries: &[FileEntry]) -> rusqlite::Result<usize> {
    with_conn(|conn| {
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO files (path, name, ext, parent, size, modified)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(path) DO UPDATE SET
                    name     = excluded.name,
                    ext      = excluded.ext,
                    parent   = excluded.parent,
                    size     = excluded.size,
                    modified = excluded.modified",
            )?;
            for e in entries {
                stmt.execute(params![
                    e.path,
                    e.name,
                    e.ext,
                    e.parent,
                    e.size,
                    e.modified
                ])?;
            }
        }
        tx.commit()?;
        Ok(entries.len())
    })
}

/// Remove a single file from the index by exact path.
pub fn delete_path(path: &str) -> rusqlite::Result<usize> {
    with_conn(|conn| conn.execute("DELETE FROM files WHERE path = ?1", params![path]))
}

/// Remove every entry under a folder prefix (used when a watched folder is removed).
pub fn delete_prefix(prefix: &str) -> rusqlite::Result<usize> {
    // Escape LIKE wildcards so paths containing `_`, `%` or `\` are literal.
    let escaped = prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("{}%", escaped);
    with_conn(|conn| {
        conn.execute(
            "DELETE FROM files WHERE path LIKE ?1 ESCAPE '\\'",
            params![pattern],
        )
    })
}

/// Number of indexed files.
pub fn count() -> rusqlite::Result<i64> {
    with_conn(|conn| conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0)))
}
