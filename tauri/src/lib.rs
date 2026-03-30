mod commands;
mod dto;
mod state;

use state::AppState;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::scan::start_scan,
            commands::scan::cancel_scan,
            commands::scan::get_scan_result,
            commands::clean::start_clean,
            commands::clean::cancel_clean,
            commands::cache::detect_caches,
            commands::cache::clean_cache,
            commands::cleaners::detect_cleaners,
            commands::config::get_config,
            commands::config::save_config,
            commands::system::get_disk_info,
            commands::system::get_app_version,
            commands::system::check_fda_status,
        ])
        .setup(|app| {
            // System tray menu
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "open", "Open null-e", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "quit", "Quit null-e", true, None::<&str>)?,
                ],
            )?;

            // Tray icon — use embedded tray-icon.png as template image
            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;

            TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("null-e")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        // Intercept window close → hide instead of quit
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Keep app running when all windows are closed
    app.run(|_app, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}
