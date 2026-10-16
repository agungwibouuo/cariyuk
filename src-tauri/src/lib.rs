mod commands;
mod db;
mod hotkey;
mod indexer;
mod search;
mod settings;
mod tray;

use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // Prevent app from exiting when windows are closed.
                // Both search and settings just hide, keeping the app alive in the tray.
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Focused(false) => {
                // Spotlight behavior: auto-hide the search panel when clicking outside.
                if window.label() == "search" {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // If another instance launches, show our search window instead.
            hotkey::toggle_search_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Initialize the index database & start the indexer.
            db::init().expect("failed to initialize search database");
            indexer::start(Some(app.handle().clone())).expect("failed to start file indexer");

            // Setup system tray.
            tray::setup(app.handle())?;

            // Register global hotkey from settings (defaults to Alt+Space).
            // If the preferred combo is taken (e.g. Windows reserves Alt+Space
            // for the system menu), fall back and persist the working combo so
            // the settings page shows the truth.
            let mut current_settings = settings::load();
            match hotkey::register_with_fallback(app.handle(), &current_settings.hotkey) {
                Ok(actual) => {
                    if actual != current_settings.hotkey {
                        eprintln!(
                            "Hotkey '{}' tidak tersedia, memakai '{}'",
                            current_settings.hotkey, actual
                        );
                        current_settings.hotkey = actual;
                        let _ = settings::save(&current_settings);
                    }
                }
                Err(e) => eprintln!("Warning: {e}"),
            }

            // Hide the search window on startup; it is shown via the global hotkey or tray.
            if let Some(window) = app.get_webview_window("search") {
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_files,
            commands::recent_files,
            commands::index_stats,
            commands::load_settings,
            commands::save_settings,
            commands::open_path,
            commands::reveal_path,
            commands::hide_search_window,
            commands::show_search_window,
            commands::open_settings_window,
            commands::reindex_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
