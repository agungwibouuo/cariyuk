import { useEffect, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { getIndexStats, hideSearchWindow, IndexStats, openFile, openSettingsWindow, revealFile } from "../lib/api";
import { useSearch } from "../hooks/useSearch";
import ResultList from "../components/ResultList";
import SearchBar from "../components/SearchBar";

export default function Search() {
  const { query, setQuery, results, isRecent } = useSearch();
  const [selected, setSelected] = useState(0);
  const [stats, setStats] = useState<IndexStats>({ count: 0, indexing: false });

  useEffect(() => {
    setSelected(0);
  }, [results]);

  // Reset selection when the panel is re-opened via hotkey/tray.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("panel-shown", () => setSelected(0)).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    let active = true;
    const tick = () => getIndexStats().then((s) => active && setStats(s));
    tick();
    const id = setInterval(tick, 3000);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, []);

  // Dynamic window height: compact when empty/short, taller when many results.
  // Spotlight-style: grow downward from an anchored top position.
  useEffect(() => {
    const items = results.length > 0 ? results.length : 1;
    const ITEM_H = 52;
    const MIN_H = 150;
    const MAX_H = 540;
    const targetH = Math.min(Math.max(MIN_H, 110 + items * ITEM_H), MAX_H);
    getCurrentWindow()
      .setSize(new LogicalSize(720, targetH))
      .catch(() => {});
  }, [results.length]);

  function handleOpenCurrent(ctrlKey: boolean) {
    const hit = results[selected];
    if (!hit) return;
    if (ctrlKey) {
      revealFile(hit.path);
    } else {
      openFile(hit.path);
    }
  }

  function handleArrow(delta: number) {
    if (results.length === 0) return;
    setSelected((prev) => {
      const next = prev + delta;
      if (next < 0) return results.length - 1;
      if (next >= results.length) return 0;
      return next;
    });
  }

  return (
    <main className="search-panel">
      <SearchBar
        query={query}
        onChange={setQuery}
        onSubmit={({ ctrlKey }) => handleOpenCurrent(ctrlKey)}
        onEscape={hideSearchWindow}
        onArrowDown={() => handleArrow(1)}
        onArrowUp={() => handleArrow(-1)}
        onFocusSettings={openSettingsWindow}
      />
      {results.length > 0 ? (
        <ResultList
          results={results}
          selected={selected}
          query={query}
          onSelect={setSelected}
          onOpen={(hit) => openFile(hit.path)}
          onReveal={(hit) => revealFile(hit.path)}
        />
      ) : (
        <div className="empty-state">
          {isRecent
            ? stats.indexing
              ? "Sedang meng-index file..."
              : `Belum ada file ter-index (${stats.count})`
            : "Tidak ada hasil"}
        </div>
      )}
      <div className="panel-footer">
        <span>{stats.indexing ? "Indexing..." : `${stats.count} file ter-index`}</span>
        <span className="hints">
          <kbd>Enter</kbd> buka · <kbd>Ctrl+Enter</kbd> folder · <kbd>Esc</kbd> tutup
        </span>
      </div>
    </main>
  );
}
