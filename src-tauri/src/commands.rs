use crate::application::eviction;
use crate::{
    application::ports::{SharedConfigStore, SharedImageStore, SharedStore},
    domain::{clip_item::ClipItem, config::Config, entry::Entry, history::History},
};
use std::borrow::Cow;
use std::sync::Mutex;
use tauri::{AppHandle, LogicalSize, Manager, State};

/// Межі вікна, спільні для ручного вводу і для збереження після ресайзу.
const MIN_W: f64 = 420.0;
const MAX_W: f64 = 2000.0;
const MIN_H: f64 = 320.0;
const MAX_H: f64 = 1600.0;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use uuid::Uuid;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewItem {
    id: String,
    is_pinned: bool,
    kind: String, //"text"|"image"
    text: String,
    thumb_path: Option<String>,
    full_path: Option<String>,
}

impl PreviewItem {
    pub fn new(entry: &Entry, is_pinned: bool, images: &SharedImageStore) -> PreviewItem {
        let (kind, thumb_path, full_path) = match entry.item() {
            ClipItem::Text(_) => (String::from("text"), None, None),
            ClipItem::Image { hash, .. } => (
                String::from("image"),
                Some(images.thumb_path(hash).to_string_lossy().into_owned()),
                Some(images.original_path(hash).to_string_lossy().into_owned()),
            ),
        };

        PreviewItem {
            id: entry.id().to_string(),
            is_pinned,
            kind,
            text: entry.preview(),
            thumb_path,
            full_path,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDto {
    recent_limit: usize,
    hotkey: String,
    window_width: f64,
    window_height: f64,
    resizable: bool,
    locale: String,
    shortcuts: std::collections::HashMap<String, String>,
}

impl ConfigDto {
    pub fn new(config: &Config) -> ConfigDto {
        let (window_width, window_height) = config.window_size();
        ConfigDto {
            recent_limit: config.recent_limit(),
            hotkey: String::from(config.hotkey()),
            window_width,
            window_height,
            resizable: config.resizable(),
            locale: String::from(config.locale()),
            shortcuts: config.shortcuts().clone(),
        }
    }
}

#[tauri::command]
pub fn get_previews(
    state: State<Mutex<History>>,
    images: State<SharedImageStore>,
) -> Vec<PreviewItem> {
    let history = state.lock().unwrap();

    history
        .pinned()
        .iter()
        .map(|i| PreviewItem::new(i, true, &images))
        .chain(
            history
                .recent()
                .iter()
                .map(|i| PreviewItem::new(i, false, &images)),
        )
        .collect()
}

#[tauri::command]
pub fn copy(
    id: String,
    state: State<Mutex<History>>,
    images: State<SharedImageStore>,
) -> Result<(), String> {
    let history = state.lock().unwrap();

    let result = history
        .pinned()
        .iter()
        .chain(history.recent())
        .find(|i| i.id().to_string() == id);

    let Some(entry) = result else {
        return Ok(());
    };

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("connect clipboard failed: {e}"))?;

    match entry.item() {
        ClipItem::Text(text) => {
            clipboard
                .set_text(text)
                .map_err(|e| format!("set clipboard failed: {e}"))?;
        }
        ClipItem::Image { hash, .. } => {
            let (rgba, width, height) = images
                .load_rgba(hash)
                .map_err(|e| format!("load image failed: {e}"))?;

            clipboard
                .set_image(arboard::ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: Cow::Owned(rgba),
                })
                .map_err(|e| format!("set image failed: {e}"))?;
        }
    };

    Ok(())
}

#[tauri::command]
pub fn pin(
    id: String,
    state: State<Mutex<History>>,
    store: State<SharedStore>,
) -> Result<(), String> {
    let mut history = state.lock().unwrap();

    let uuid = Uuid::parse_str(&id).map_err(|e| format!("uuid parse failed: {e}"))?;
    history.pin(uuid);

    if let Err(e) = store.save(&history) {
        return Err(format!("save failed: {e}"));
    }

    Ok(())
}

#[tauri::command]
pub fn remove_pinned(
    id: String,
    history: State<Mutex<History>>,
    store: State<SharedStore>,
    images: State<SharedImageStore>,
) -> Result<(), String> {
    let mut hstr = history.lock().unwrap();

    let uuid = Uuid::parse_str(&id).map_err(|e| format!("uuid parse failed: {e}"))?;
    let evicted = hstr.remove_pinned(uuid);

    eviction::forget_entries(evicted, &images);

    if let Err(e) = store.save(&hstr) {
        return Err(format!("save failed: {e}"));
    }

    Ok(())
}

#[tauri::command]
pub fn set_recent_limit(
    limit: usize,
    config: State<Mutex<Config>>,
    history: State<Mutex<History>>,
    config_store: State<SharedConfigStore>,
    store: State<SharedStore>,
    images: State<SharedImageStore>,
) -> Result<(), String> {
    let mut cfg = config.lock().unwrap();
    cfg.set_recent_limit(limit);

    if let Err(e) = config_store.save(&cfg) {
        return Err(format!("save recent limit failed: {e}"));
    }

    let mut hstr = history.lock().unwrap();
    let evicteds = hstr.set_recent_limit(limit);

    eviction::forget_entries(evicteds, &images);

    if let Err(e) = store.save(&hstr) {
        return Err(format!("save history recent limit failed: {e}"));
    }

    Ok(())
}

#[tauri::command]
pub fn set_hotkey(
    hotkey: String,
    app: AppHandle,
    config: State<Mutex<Config>>,
    config_store: State<SharedConfigStore>,
) -> Result<(), String> {
    let new: Shortcut = hotkey
        .parse()
        .map_err(|e| format!("parse new hotkey failed: {e}"))?;
    let mut cfg = config.lock().unwrap();
    let old: Shortcut = cfg.hotkey().parse().expect("Old hotckey exist");

    let _ = app.global_shortcut().unregister(old);
    if let Err(e) = app.global_shortcut().register(new) {
        if let Err(rollback_err) = app.global_shortcut().register(old) {
            eprintln!("відкат не вдався, хоткея нема зовсім: {rollback_err}");
        }
        return Err(format!("register new hotkey failed: {e}"));
    }

    cfg.set_hotkey(hotkey);
    if let Err(e) = config_store.save(&cfg) {
        return Err(format!("save hotkey failed: {e}"));
    }
    Ok(())
}

#[tauri::command]
pub fn get_config(config: State<Mutex<Config>>) -> ConfigDto {
    let cfg = config.lock().unwrap();

    ConfigDto::new(&cfg)
}

/// Спільне для команд, що правлять конфіг: зберегти й перекласти помилку для UI.
fn persist(config: &Config, store: &SharedConfigStore) -> Result<(), String> {
    store
        .save(config)
        .map_err(|e| format!("save config failed: {e}"))
}

#[tauri::command]
pub fn set_window_size(
    width: f64,
    height: f64,
    app: AppHandle,
    config: State<Mutex<Config>>,
    config_store: State<SharedConfigStore>,
) -> Result<(), String> {
    if !(MIN_W..=MAX_W).contains(&width) || !(MIN_H..=MAX_H).contains(&height) {
        return Err(format!(
            "розмір поза межами: {MIN_W}×{MIN_H} … {MAX_W}×{MAX_H}"
        ));
    }

    if let Some(window) = app.get_webview_window("main") {
        window
            .set_size(LogicalSize::new(width, height))
            .map_err(|e| format!("set window size failed: {e}"))?;
    }

    let mut cfg = config.lock().unwrap();
    cfg.set_window_size(width, height);
    persist(&cfg, &config_store)
}

#[tauri::command]
pub fn set_resizable(
    resizable: bool,
    app: AppHandle,
    config: State<Mutex<Config>>,
    config_store: State<SharedConfigStore>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_resizable(resizable)
            .map_err(|e| format!("set resizable failed: {e}"))?;
    }

    let mut cfg = config.lock().unwrap();
    cfg.set_resizable(resizable);
    persist(&cfg, &config_store)
}

#[tauri::command]
pub fn set_locale(
    locale: String,
    config: State<Mutex<Config>>,
    config_store: State<SharedConfigStore>,
) -> Result<(), String> {
    let mut cfg = config.lock().unwrap();
    cfg.set_locale(locale);
    persist(&cfg, &config_store)
}

#[tauri::command]
pub fn set_shortcut(
    action: String,
    combination: Option<String>,
    config: State<Mutex<Config>>,
    config_store: State<SharedConfigStore>,
) -> Result<(), String> {
    let mut cfg = config.lock().unwrap();
    cfg.set_shortcut(action, combination);
    persist(&cfg, &config_store)
}
