import { useEffect, useRef, useState } from "react";
import { FileHit, searchFiles, recentFiles } from "../lib/api";

/**
 * Debounced file search. Returns the current results, whether a query is
 * active, and a flag for the "no query" (recent files) state.
 */
export function useSearch(debounceMs = 80) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<FileHit[]>([]);
  const [loading, setLoading] = useState(false);
  const seq = useRef(0);

  useEffect(() => {
    const trimmed = query.trim();
    const id = ++seq.current;

    const timer = setTimeout(async () => {
      setLoading(true);
      let hits: FileHit[];
      if (trimmed === "") {
        hits = await recentFiles();
      } else {
        hits = await searchFiles(trimmed);
      }
      // Ignore stale responses if a newer query was issued.
      if (id === seq.current) {
        setResults(hits);
        setLoading(false);
      }
    }, debounceMs);

    return () => clearTimeout(timer);
  }, [query, debounceMs]);

  return { query, setQuery, results, loading, isRecent: query.trim() === "" };
}
