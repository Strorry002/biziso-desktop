// Biziso desktop shell. The main window loads the Biziso web platform
// (configured in tauri.conf.json); the native layer adds the value a browser
// tab cannot.
//
// F0: single-instance, persisted window state.
// F1: system tray + minimize-to-tray.
//     (The web-driven layer -- unread badge + notifications over a postMessage
//     bridge -- is F1.2; biziso:// deep links and auto-update follow.)

#[cfg(desktop)]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri::Manager;

fn show_main(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        // Single-instance must be registered first so a second launch is
        // intercepted before a window is created; focus the running one.
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                show_main(app);
            }))
            .plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                // System tray: left-click shows the window; the menu has Show / Quit.
                let show_i = MenuItem::with_id(app, "show", "Show Biziso", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
                let _tray = TrayIconBuilder::with_id("main")
                    .icon(app.default_window_icon().expect("bundled icon").clone())
                    .tooltip("Biziso")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => show_main(app),
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
                            show_main(tray.app_handle());
                        }
                    })
                    .build(app)?;

                // Minimize-to-tray: closing the window hides it, keeping the app
                // alive in the tray. Quit is via the tray menu.
                if let Some(win) = app.get_webview_window("main") {
                    let w = win.clone();
                    win.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            let _ = w.hide();
                        }
                    });
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Biziso desktop");
}
