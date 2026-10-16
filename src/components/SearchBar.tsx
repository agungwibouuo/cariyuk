import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface SearchBarProps {
  query: string;
  onChange: (value: string) => void;
  onSubmit: (e: { ctrlKey: boolean }) => void;
  onEscape: () => void;
  onArrowUp: () => void;
  onArrowDown: () => void;
  onFocusSettings: () => void;
}

export default function SearchBar({
  query,
  onChange,
  onSubmit,
  onEscape,
  onArrowUp,
  onArrowDown,
  onFocusSettings,
}: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();

    let unlisten: (() => void) | undefined;
    const win = getCurrentWindow();
    win.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, []);

  return (
    <div className="search-bar">
      <span className="search-icon">🔍</span>
      <input
        ref={inputRef}
        type="text"
        value={query}
        placeholder="Cari file..."
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") onSubmit({ ctrlKey: e.ctrlKey });
          if (e.key === "Escape") onEscape();
          if (e.key === "ArrowDown") {
            e.preventDefault();
            onArrowDown();
          }
          if (e.key === "ArrowUp") {
            e.preventDefault();
            onArrowUp();
          }
        }}
      />
      <button className="settings-trigger" title="Pengaturan" onClick={onFocusSettings}>
        ⚙
      </button>
    </div>
  );
}
