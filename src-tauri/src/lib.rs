// Biziso desktop shell. The main window loads the Biziso web platform
// (configured in tauri.conf.json); the native layer adds the value a browser
// tab cannot.
//
// F0:   single-instance, persisted window state.
// F1:   system tray + minimize-to-tray.
// F1.2: web -> native bridge (unread tray badge + OS notifications via events).
// F2:   biziso:// deep links -- focus the window and navigate the webview to
//       the matching biziso.com URL (auth/login return, message links).

#[cfg(desktop)]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri::{Listener, Manager};
#[cfg(desktop)]
use tauri_plugin_deep_link::DeepLinkExt;

// Injected after every page load. Turns mod-touch's postMessages into Tauri
// events the Rust side listens for. mod-touch stays Tauri-agnostic.
const BRIDGE_JS: &str = r#"
(function () {
  if (window.__bizisoBridge) return;
  window.__bizisoBridge = true;
  function emit(ev, payload) {
    try {
      if (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.emit) {
        window.__TAURI__.event.emit(ev, payload);
      }
    } catch (e) {}
  }
  window.addEventListener('message', function (e) {
    var d = e && e.data;
    if (!d || typeof d !== 'object') return;
    if (d.type === 'biziso.unread') {
      emit('biziso://unread', { count: Math.max(0, parseInt(d.count, 10) || 0) });
    } else if (d.type === 'biziso.notify') {
      emit('biziso://notify', { title: String(d.title || 'Biziso'), body: String(d.body || '') });
    }
  });
})();
"#;

fn apply_unread(app: &tauri::AppHandle, count: u32) {
    let label = if count > 0 {
        format!("Biziso ({})", count)
    } else {
        "Biziso".to_string()
    };
    #[cfg(desktop)]
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(&label));
    }
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_title(&label);
    }
}

fn do_notify(app: &tauri::AppHandle, title: String, body: String) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

// Focus the window and navigate the webview to the biziso.com URL a biziso://
// deep link maps to (biziso://<path> -> https://biziso.com/<path>).
fn handle_deep_link(app: &tauri::AppHandle, url: &str) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
        if let Some(rest) = url.strip_prefix("biziso://") {
            let target = format!("https://biziso.com/{}", rest.trim_start_matches('/'));
            let js = format!(
                "window.location.assign({})",
                serde_json::to_string(&target).unwrap_or_else(|_| "\"https://biziso.com\"".to_string())
            );
            let _ = win.eval(&js);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
                show_main(app);
                // On Windows a deep link wakes the running app via a second
                // launch; the biziso:// URL arrives in argv.
                if let Some(url) = args.iter().find(|a| a.starts_with("biziso://")) {
                    handle_deep_link(app, url);
                }
            }))
            .plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .on_page_load(|webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let _ = webview.eval(BRIDGE_JS);
            }
        })
        .setup(|app| {
            // Bridge: the web app emits these events; we drive the native layer.
            let h = app.handle().clone();
            app.listen("biziso://unread", move |event| {
                let count = serde_json::from_str::<serde_json::Value>(event.payload())
                    .ok()
                    .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
                    .unwrap_or(0) as u32;
                apply_unread(&h, count);
            });
            let h2 = app.handle().clone();
            app.listen("biziso://notify", move |event| {
                let v = serde_json::from_str::<serde_json::Value>(event.payload())
                    .unwrap_or(serde_json::Value::Null);
                let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("Biziso").to_string();
                let body = v.get("body").and_then(|b| b.as_str()).unwrap_or("").to_string();
                do_notify(&h2, title, body);
            });

            #[cfg(desktop)]
            {
                // Deep links: register the scheme at runtime (needed in dev; the
                // installer registers it for packaged builds) + handle opens.
                #[cfg(any(windows, target_os = "linux"))]
                let _ = app.deep_link().register_all();
                let dh = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        handle_deep_link(&dh, url.as_str());
                    }
                });

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
