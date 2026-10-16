use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Show + focus the search window, or hide it when it is already in front.
pub fn toggle_search_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("search") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);
        if visible && focused {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            // Let the frontend reset its selection state.
            let _ = app.emit_to("search", "panel-shown", ());
        }
    }
}

/// Fallback chain used when the preferred hotkey is taken by the OS or another
/// app. `Alt+Space` in particular is reserved by Windows for the window system
/// menu, so it often fails on a fresh machine.
pub const FALLBACK_HOTKEYS: [&str; 3] = ["Ctrl+Alt+Space", "Ctrl+Shift+Space", "Alt+Shift+Space"];

/// Register the preferred hotkey, falling back to a known-free combo when it is
/// already taken. Returns the hotkey that was actually registered.
pub fn register_with_fallback(app: &AppHandle, preferred: &str) -> Result<String, String> {
    if register_hotkey(app, preferred).is_ok() {
        return Ok(preferred.to_string());
    }

    for candidate in FALLBACK_HOTKEYS {
        if candidate != preferred && register_hotkey(app, candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    Err(format!(
        "tidak ada hotkey yang bisa didaftarkan (dicoba '{preferred}' dan {} alternatif)",
        FALLBACK_HOTKEYS.len()
    ))
}

/// (Re-)register the global hotkey from settings. Any previous registration is
/// removed first, so the settings page can swap hotkeys freely.
pub fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    shortcuts
        .on_shortcut(hotkey, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_search_window(app);
            }
        })
        .map_err(|e| format!("gagal mendaftarkan hotkey '{hotkey}': {e}"))
}
