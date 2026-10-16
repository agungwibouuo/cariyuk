use crate::db::{self, FileEntry};
use crate::settings;
use notify::RecursiveMode;
use notify::Watcher;
use notify_debouncer_full::{new_debouncer, DebouncedEvent};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::Emitter;
use walkdir::WalkDir;

/// Whether the initial scan is currently running.
static IS_INDEXING: AtomicBool = AtomicBool::new(false);

/// Folders to ignore during indexing.
const IGNORED_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    "$RECYCLE.BIN",
    "System Volume Information",
    ".cache",
    "AppData",
];

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy();
    if file_name.starts_with('.') && file_name.len() > 1 {
        return true;
    }
    for ignored in IGNORED_NAMES {
        if file_name.eq_ignore_ascii_case(ignored) {
            return true;
        }
    }
    false
}

fn file_to_entry(path: &Path) -> Option<FileEntry> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().to_string();
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let size = meta.len() as i64;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    Some(FileEntry {
        path: path.to_string_lossy().to_string(),
        name,
        ext,
        parent,
        size,
        modified,
    })
}

/// Recursively scan a folder and insert into DB in batches.
fn scan_folder(folder_path: &Path) {
    let mut batch: Vec<FileEntry> = Vec::with_capacity(500);

    for entry in WalkDir::new(folder_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip(e))
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(file_entry) = file_to_entry(entry.path()) {
                batch.push(file_entry);
                if batch.len() >= 500 {
                    let _ = db::upsert_many(&batch);
                    batch.clear();
                }
            }
        }
    }

    if !batch.is_empty() {
        let _ = db::upsert_many(&batch);
    }
}

/// Handle file system change events from the watcher.
fn handle_events(events: Vec<DebouncedEvent>) {
    for debounced in events {
        for path in &debounced.paths {
            if path.is_file() {
                if let Some(entry) = file_to_entry(path) {
                    let _ = db::upsert_many(&[entry]);
                }
            } else if !path.exists() {
                // File or folder was removed.
                let path_str = path.to_string_lossy();
                let _ = db::delete_path(&path_str);
                // When a folder is removed, also drop its children (prefix + separator
                // so that `C:\Docs` does not match `C:\Docs2`).
                let folder_prefix = format!("{}{}", path_str, std::path::MAIN_SEPARATOR);
                let _ = db::delete_prefix(&folder_prefix);
            }
        }
    }
}

/// Start indexing and watching configured folders in the background.
pub fn start(app: Option<tauri::AppHandle>) -> Result<(), Box<dyn std::error::Error>> {
    let config = settings::load();
    let folders: Vec<PathBuf> = config
        .indexed_folders
        .into_iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    if folders.is_empty() {
        return Ok(());
    }

    // Channel between the watcher callback and the event-processing thread.
    let (tx, rx) = std::sync::mpsc::channel::<Vec<DebouncedEvent>>();
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |res: notify_debouncer_full::DebounceEventResult| {
            if let Ok(events) = res {
                let _ = tx.send(events);
            }
        },
    )?;

    for folder in &folders {
        debouncer
            .watcher()
            .watch(folder, RecursiveMode::Recursive)?;
    }

    // Keep the debouncer alive by moving it into the event-processing thread.
    std::thread::spawn(move || {
        // `debouncer` lives as long as this thread.
        while let Ok(events) = rx.recv() {
            handle_events(events);
        }
        // Receiver dropped / sender closed → thread exits, debouncer dropped.
    });

    // Background thread for the initial scan (does not block app launch).
    let scan_folders = folders.clone();
    std::thread::spawn(move || {
        if IS_INDEXING.swap(true, Ordering::SeqCst) {
            return; // Already indexing
        }
        for folder in &scan_folders {
            scan_folder(folder);
        }
        IS_INDEXING.store(false, Ordering::SeqCst);

        // Notify the frontend that the initial scan has finished.
        if let Some(app) = app {
            let _ = app.emit("index-ready", db::count().unwrap_or(0));
        }
    });

    Ok(())
}

/// Trigger a rescan of configured folders (e.g. from Tray menu or Settings).
pub fn rescan(app: Option<tauri::AppHandle>) {
    let config = settings::load();
    let folders: Vec<PathBuf> = config
        .indexed_folders
        .into_iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    std::thread::spawn(move || {
        if IS_INDEXING.swap(true, Ordering::SeqCst) {
            return;
        }
        for folder in &folders {
            scan_folder(folder);
        }
        IS_INDEXING.store(false, Ordering::SeqCst);

        if let Some(app) = app {
            let _ = app.emit("index-ready", db::count().unwrap_or(0));
        }
    });
}

/// Check if indexer is currently performing the initial scan.
pub fn is_indexing() -> bool {
    IS_INDEXING.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that scan_folder correctly indexes files under a temporary folder.
    #[test]
    fn scan_folder_indexes_files() {
        crate::db::init().expect("db init failed");

        // Build a temporary folder with a few files.
        let tmp_dir = std::env::temp_dir().join(format!(
            "cariyuk_test_{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&tmp_dir);

        let file1 = tmp_dir.join("hello.txt");
        let file2 = tmp_dir.join("report_2026.pdf");
        let _ = std::fs::write(&file1, "hi");
        let _ = std::fs::write(&file2, "%PDF fake");

        // Scan it.
        scan_folder(&tmp_dir);

        // Both files should be indexed.
        let count = crate::db::count().expect("count failed");
        assert!(count >= 2, "expected at least 2 files, got {count}");

        // Cleanup: remove them from the DB and on disk.
        let _ = crate::db::delete_prefix(&tmp_dir.to_string_lossy());
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
