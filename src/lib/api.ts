import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

/** A single file search result coming from the Rust backend. */
export interface FileHit {
  path: string;
  name: string;
  ext: string | null;
  parent: string;
  size: number;
  modified: number;
  /** Fuzzy score, higher is better. */
  score: number;
}

/** Persisted app settings. */
export interface AppSettings {
  indexedFolders: string[];
  hotkey: string;
  autostart: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  indexedFolders: [],
  hotkey: "Alt+Space",
  autostart: false,
};

/**
 * Search indexed files by query. Backend does fuzzy ranking and returns
 * results already sorted by relevance.
 */
export async function searchFiles(query: string, limit = 50): Promise<FileHit[]> {
  try {
    return await invoke<FileHit[]>("search_files", { query, limit });
  } catch {
    return [];
  }
}

/** Most recently modified files, shown when the query is empty. */
export async function recentFiles(limit = 20): Promise<FileHit[]> {
  try {
    return await invoke<FileHit[]>("recent_files", { limit });
  } catch {
    return [];
  }
}

export interface IndexStats {
  count: number;
  indexing: boolean;
}

export async function getIndexStats(): Promise<IndexStats> {
  try {
    return await invoke<IndexStats>("index_stats");
  } catch {
    return { count: 0, indexing: false };
  }
}

export async function loadSettings(): Promise<AppSettings> {
  try {
    return await invoke<AppSettings>("load_settings");
  } catch {
    return DEFAULT_SETTINGS;
  }
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  await invoke("save_settings", { settings });
}

/** Open a file with the OS default application. */
export async function openFile(path: string): Promise<void> {
  await invoke("open_path", { path });
}

/** Reveal a file in the OS file manager. */
export async function revealFile(path: string): Promise<void> {
  await invoke("reveal_path", { path });
}

/** Hide the search window (called on Escape). */
export async function hideSearchWindow(): Promise<void> {
  await invoke("hide_search_window");
}

/** Open the Settings window. */
export async function openSettingsWindow(): Promise<void> {
  await invoke("open_settings_window");
}

/** Open a native folder-picker dialog; returns the chosen path or `null` on cancel. */
export async function pickFolder(): Promise<string | null> {
  const result = await openDialog({
    directory: true,
    multiple: false,
    title: "Pilih Folder untuk Di-index",
  });
  return typeof result === "string" ? result : null;
}

/** Trigger a full re-scan of all configured folders. */
export async function reindexNow(): Promise<void> {
  await invoke("reindex_now");
}
