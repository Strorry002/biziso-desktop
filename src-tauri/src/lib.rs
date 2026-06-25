// Biziso desktop shell. The main window loads the Biziso web platform
// (configured in tauri.conf.json); the native layer here adds the value a
// browser tab cannot. F0 covers single-instance and persisted window state;
// tray, notifications, deep links and auto-update arrive in later phases.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        // Single-instance must be registered first so a second launch is
        // intercepted before a window is created; focus the running one.
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running Biziso desktop");
}
