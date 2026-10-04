// Just Clipboard Manager — clipboard history for macOS
// Copyright (C) 2026 Vlad Kutsa
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version. See the LICENSE file for details.

mod application;
mod commands;
mod domain;
mod infrastructure;
mod watcher;
use crate::application::ports::ConfigStore;
use crate::application::ports::SharedConfigStore;
use crate::application::ports::SharedImageStore;
use crate::application::ports::SharedStore;
use crate::infrastructure::fs_image_store::FsImageStore;
use crate::infrastructure::json_config_store::JsonConfigStore;
use crate::infrastructure::json_store::JsonStore;
use domain::config::Config;
use domain::history::History;
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_nspanel::ManagerExt;

tauri_nspanel::tauri_panel! {
    panel!(ClipboardPanel {
        config: {
            can_become_key_window: true
        }
    })
}

/// Записує поточний розмір вікна в конфіг у пам'яті (без диска).
fn remember_window_size(window: &tauri::Window) {
    let Ok(size) = window.inner_size() else {
        return;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let logical = size.to_logical::<f64>(scale);
    if logical.width <= 0.0 || logical.height <= 0.0 {
        return; // згорнуте вікно віддає нулі — такий «розмір» зберігати не можна
    }

    let state = window.state::<Mutex<Config>>();
    let mut config = state.lock().unwrap();
    config.set_window_size(logical.width, logical.height);
}

/// Те саме, але з записом на диск.
fn persist_window_size(window: &tauri::Window) {
    remember_window_size(window);

    let config = window.state::<Mutex<Config>>();
    let store = window.state::<SharedConfigStore>();
    let config = config.lock().unwrap();
    if let Err(e) = store.save(&config) {
        eprintln!("save window size failed: {e}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_nspanel::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            //CONFIG
            let config_file: String = String::from("config.json");
            let config_path = dir.join(config_file);
            let config_store: SharedConfigStore = Arc::new(JsonConfigStore::new(config_path));
            let config = config_store.load().unwrap_or_default();
            let shortcut: Shortcut = config.hotkey().parse().unwrap_or_else(|e| {
                eprintln!("невалідний хоткей у конфізі: {e}, беру дефолтний");
                Config::default()
                    .hotkey()
                    .parse()
                    .expect("дефолтний хоткей у Config::default() має бути валідним")
            });
            //HISTORY LIST
            let file: String = String::from("history.json");
            let path = dir.join(file);
            let store: SharedStore = Arc::new(JsonStore::new(path));
            let mut history = store
                .load()
                .unwrap_or_else(|_| History::new(config.recent_limit()));
            history.set_recent_limit(config.recent_limit());
            if let Some(window) = app.get_webview_window("main") {
                let (w, h) = config.window_size();
                let _ = window.set_size(tauri::LogicalSize::new(w, h));
                let _ = window.set_resizable(config.resizable());
            }

            let images: SharedImageStore = Arc::new(FsImageStore::new(dir.join("images")));

            if let Err(e) = images.cleanup(&history.image_hashes()) {
                eprintln!("images cleanup failed: {e}");
            };
            
            app.manage(Mutex::new(config));
            app.manage(Mutex::new(history));
            app.manage(config_store.clone());
            app.manage(store.clone());
            app.manage(images.clone());
            watcher::start(app.handle().clone(), store.clone(), images);

            #[cfg(desktop)]
            {
                // let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |app, _shortcut, event| {
                            if event.state() != ShortcutState::Pressed {
                                return;
                            }

                            toggle_window(&app);
                        })
                        .build(),
                )?;
                app.global_shortcut().register(shortcut)?;

                let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show, &quit])?;
                // macOS малює меню-бар іконки як template image: чорна фігура
                // з альфою, колір підставляє система (білий у темній темі,
                // чорний у світлій). Тому тут окрема іконка, не віконна.
                let tray_icon = tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray.png"
                ))?;
                TrayIconBuilder::new()
                    .icon(tray_icon)
                    .icon_as_template(true)
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "show" => toggle_window(&app),
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .build(app)?;
            }

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                use tauri_nspanel::objc2_app_kit::{NSWindowCollectionBehavior, NSWindowStyleMask};
                use tauri_nspanel::{WebviewWindowExt};

                if let Some(window) = app.get_webview_window("main") {
                    let panel = window.to_panel::<ClipboardPanel>()?;
                    panel
                        .add_style_mask(NSWindowStyleMask::NonactivatingPanel)
                        .ok();
                    panel.set_level(24); // рівень menu bar
                    panel.set_collection_behavior(
                        NSWindowCollectionBehavior::CanJoinAllSpaces
                            | NSWindowCollectionBehavior::FullScreenAuxiliary,
                    );
                }
            }

            Ok(()) //setup must return Result
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                persist_window_size(window);
                let _ = window.hide();
            }
            // Resized прилітає на КОЖЕН піксель перетягування — у файл на кожен
            // із них писати не можна, тож тут лише оновлюємо конфіг у пам'яті.
            tauri::WindowEvent::Resized(_) => remember_window_size(window),
            // а на диск скидаємо тоді, коли людина вже відпустила вікно
            tauri::WindowEvent::Focused(false) => persist_window_size(window),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::copy,
            commands::pin,
            commands::remove_pinned,
            commands::get_previews,
            commands::get_config,
            commands::set_recent_limit,
            commands::set_hotkey,
            commands::set_window_size,
            commands::set_resizable,
            commands::set_locale,
            commands::set_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_window(app: &AppHandle) {
    let Ok(panel) = app.get_webview_panel("main") else { return; };

    if panel.is_visible() {
        panel.hide();
    } else {
        panel.show_and_make_key();
    }
}
