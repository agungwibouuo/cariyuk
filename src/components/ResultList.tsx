import { useEffect, useRef } from "react";
import { FileHit } from "../lib/api";
import FileTypeIcon from "./FileTypeIcon";

interface ResultListProps {
  results: FileHit[];
  selected: number;
  query: string;
  onSelect: (index: number) => void;
  onOpen: (hit: FileHit) => void;
  onReveal: (hit: FileHit) => void;
}

/** Split a file name into [before, match, after] for highlighting */
function highlightParts(name: string, query: string): { pre: string; hit: string; post: string } {
  const words = query
    .split(/\s+/)
    .filter((w) => w.length > 0 && !w.includes(":"));
  const lower = name.toLowerCase();
  let start = -1;
  let len = 0;
  for (const word of words) {
    const idx = lower.indexOf(word.toLowerCase());
    if (idx >= 0 && (start === -1 || idx < start)) {
      start = idx;
      len = word.length;
    }
  }
  if (start === -1) return { pre: name, hit: "", post: "" };
  return {
    pre: name.slice(0, start),
    hit: name.slice(start, start + len),
    post: name.slice(start + len),
  };
}

function formatSize(bytes: number): string {
  if (bytes <= 0) return "-";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
}

function formatRelativeTime(epochSeconds: number): string {
  if (!epochSeconds) return "";
  const now = Math.floor(Date.now() / 1000);
  const diff = Math.max(0, now - epochSeconds);

  if (diff < 60) return "baru saja";
  if (diff < 3600) return `${Math.floor(diff / 60)} mnt lalu`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} jam lalu`;
  if (diff < 172800) return "kemarin";
  if (diff < 604800) return `${Math.floor(diff / 86400)} hari lalu`;

  const d = new Date(epochSeconds * 1000);
  return d.toLocaleDateString("id-ID", {
    day: "numeric",
    month: "short",
  });
}

export default function ResultList({
  results,
  selected,
  query,
  onSelect,
  onOpen,
  onReveal,
}: ResultListProps) {
  const listRef = useRef<HTMLUListElement>(null);

  // Keep the selected item visible while navigating with keyboard.
  useEffect(() => {
    const item = listRef.current?.children[selected] as HTMLElement | undefined;
    item?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  return (
    <ul className="result-list" ref={listRef}>
      {results.map((hit, index) => {
        const { pre, hit: matched, post } = highlightParts(hit.name, query);
        const isSelected = index === selected;
        return (
          <li
            key={hit.path}
            className={`result-item ${isSelected ? "active" : ""}`}
            onMouseEnter={() => onSelect(index)}
            onClick={() => onOpen(hit)}
            onContextMenu={(e) => {
              e.preventDefault();
              onReveal(hit);
            }}
          >
            <FileTypeIcon ext={hit.ext} />
            <div className="result-main">
              <span className="result-name">
                {pre}
                {matched && <mark>{matched}</mark>}
                {post}
              </span>
              <span className="result-path" title={hit.path}>
                {hit.parent}
              </span>
            </div>
            <div className="result-meta">
              <span className="result-time">{formatRelativeTime(hit.modified)}</span>
              <span className="result-size">{formatSize(hit.size)}</span>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
