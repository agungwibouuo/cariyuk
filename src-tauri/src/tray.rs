use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let hotkey = crate::settings::load().hotkey;
    let search_item =
        MenuItem::with_id(app, "search", format!("Cari ({hotkey})"), true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Pengaturan...", true, None::<&str>)?;
    let reindex_item = MenuItem::with_id(app, "reindex", "Re-index Folder", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Keluar CariYuk", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&search_item, &settings_item, &reindex_item, &quit_item])?;

    let mut builder = TrayIconBuilder::new()
        .tooltip("CariYuk — Pencarian Cepat")
        .menu(&menu)
        .show_menu_on_left_click(false);

    // Reuse the app's default window icon for the tray.
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    // Reuse the app's default window icon for the tray.
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "search" => crate::hotkey::toggle_search_window(app),
            "settings" => crate::commands::open_settings_window(app.clone()),
            "reindex" => {
                crate::indexer::rescan(Some(app.clone()));
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::hotkey::toggle_search_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
