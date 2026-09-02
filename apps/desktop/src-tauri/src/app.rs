use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WindowEvent};

use crate::commands;

/// Builds the Tauri app: DB init, downloads watcher startup, Floating Card
/// window, system tray, and hide-to-tray on close. Startup reconciliation
/// lands in a later milestone (see docs/architecture.md).
pub fn build() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::mark_later,
            commands::groups::create_group,
            commands::groups::list_groups,
            commands::groups::assign_group,
            commands::operations::undo_operation,
            commands::temporary::mark_temporary,
            commands::temporary::list_temporary,
            commands::temporary::move_to_recycle_bin,
        ])
        .setup(|app| {
            tracing_subscriber::fmt::init();

            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("download-inbox.sqlite");

            let pool = tauri::async_runtime::block_on(storage::init_db(&db_path))?;
            let watcher_pool = pool.clone();
            let sweep_pool = pool.clone();
            app.manage(pool);

            tauri::async_runtime::spawn(crate::temporary::run(app.handle().clone(), sweep_pool));

            // Watcher failure must not take the rest of the app down with it
            // (spec section 40) — the user can still use groups/history/undo
            // even if e.g. the configured folder is missing or unwatchable.
            let downloads_dir = app.path().download_dir()?;
            if let Err(err) = std::fs::create_dir_all(&downloads_dir) {
                tracing::warn!(?err, dir = %downloads_dir.display(), "could not ensure downloads directory exists");
            }
            match crate::inbox::start(app.handle().clone(), watcher_pool, downloads_dir) {
                Ok(watcher_handle) => {
                    app.manage(watcher_handle);
                }
                Err(err) => {
                    tracing::error!(?err, "failed to start downloads watcher; continuing without it");
                }
            }

            crate::floating_card::init(app.handle())?;

            let open_item =
                MenuItem::with_id(app, "open", "Open Download Inbox", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&open_item, &quit_item])?;

            // The returned TrayIcon must be kept alive for as long as the app runs —
            // dropping it (e.g. not binding `.build(app)?`) tears down its menu/click
            // wiring immediately, so it's stashed in managed state rather than discarded.
            let tray_icon = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            app.manage(tray_icon);

            if let Some(window) = app.get_webview_window("main") {
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // Quit is only reachable from the tray menu, per spec section 28:
                        // closing the main window hides it and keeps the core running.
                        api.prevent_close();
                        let _ = window_handle.hide();
                    }
                });
            }

            Ok(())
        })
}
