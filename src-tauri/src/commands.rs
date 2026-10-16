use crate::search;
use crate::settings::{self, AppSettings};
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

/// Search indexed files by query. Returns hits sorted by relevance.
#[tauri::command]
pub fn search_files(query: String, limit: Option<usize>) -> Vec<search::FileHit> {
    let limit = limit.unwrap_or(50);
    search::search(&query, limit).unwrap_or_default()
}

/// Most recently modified files, shown when the query is empty.
#[tauri::command]
pub fn recent_files(limit: Option<usize>) -> Vec<search::FileHit> {
    let limit = limit.unwrap_or(20);
    search::recent(limit).unwrap_or_default()
}

/// Number of files currently indexed + whether the initial scan is running.
#[derive(serde::Serialize)]
pub struct IndexStats {
    pub count: i64,
    pub indexing: bool,
}

#[tauri::command]
pub fn index_stats() -> IndexStats {
    IndexStats {
        count: crate::db::count().unwrap_or(0),
        indexing: crate::indexer::is_indexing(),
    }
}

#[tauri::command]
pub fn load_settings() -> AppSettings {
    settings::load()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, app: AppHandle) -> Result<(), String> {
    // Persist first so a hotkey failure does not lose the other settings.
    let previous = settings::load();
    settings::save(&settings).map_err(|e| e.to_string())?;

    // Apply autostart preference immediately.
    let autostart = app.autolaunch();
    if settings.autostart {
        let _ = autostart.enable();
    } else {
        let _ = autostart.disable();
    }

    // Re-register the global hotkey if it changed.
    if settings.hotkey != previous.hotkey {
        match crate::hotkey::register_with_fallback(&app, &settings.hotkey) {
            Ok(actual) => {
                if actual != settings.hotkey {
                    // Persist the fallback so the UI shows the registered combo.
                    let mut fixed = settings.clone();
                    fixed.hotkey = actual.clone();
                    settings::save(&fixed).map_err(|e| e.to_string())?;
                    return Err(format!(
                        "Hotkey '{}' tidak tersedia. Didaftarkan sebagai '{}' sebagai gantinya.",
                        settings.hotkey, actual
                    ));
                }
            }
            Err(err) => {
                // Roll back to the last working hotkey so the app stays usable.
                let _ = crate::hotkey::register_hotkey(&app, &previous.hotkey);
                return Err(err);
            }
        }
    }

    // Re-index if the folder list changed.
    if settings.indexed_folders != previous.indexed_folders {
        crate::indexer::rescan(Some(app.clone()));
    }

    Ok(())
}

/// Open a file or folder with the OS default application.
#[tauri::command]
pub fn open_path(path: String, app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Re-index all configured folders from scratch (e.g. after a settings change
/// or when the user wants a clean scan).
#[tauri::command]
pub fn reindex_now(app: AppHandle) {
    crate::indexer::rescan(Some(app));
}

/// Reveal a file in the OS file manager (Explorer on Windows).
#[tauri::command]
pub fn reveal_path(path: String, app: AppHandle) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| e.to_string())
}

/// Hide the search window instead of exiting (app stays in tray).
#[tauri::command]
pub fn hide_search_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("search") {
        let _ = window.hide();
    }
}

/// Show + focus the search window (same as pressing the global hotkey).
#[tauri::command]
pub fn show_search_window(app: AppHandle) {
    crate::hotkey::toggle_search_window(&app);
}

/// Show + focus the settings window (pre-created hidden at startup).
/// Closing it only hides it, so the webview content stays loaded and the
/// window never flashes white.
#[tauri::command]
pub fn open_settings_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
