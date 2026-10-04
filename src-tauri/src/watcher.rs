use crate::application::eviction;
use crate::application::ports::{SharedImageStore, SharedStore};
use crate::domain::clip_item::ClipItem;
use crate::domain::history::History;
use arboard::Clipboard;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub fn start(app: AppHandle, store: SharedStore, images: SharedImageStore) {
    thread::spawn(move || {
        let mut clipboard = Clipboard::new().unwrap();
        let mut last: Option<ClipItem> = None;
        loop {
            let current: Option<ClipItem> = if let Ok(data) = clipboard.get_image() {
                let width = data.width as u32;
                let height = data.height as u32;
                match images.save(&data.bytes, width, height) {
                    Ok(hash) => Some(ClipItem::Image {
                        hash,
                        width,
                        height,
                    }),
                    Err(e) => {
                        eprintln!("image save failed: {e}");
                        None
                    }
                }
            } else if let Ok(text) = clipboard.get_text() {
                Some(ClipItem::Text(text))
            } else {
                None
            };

            if let Some(current) = current {
                if last.as_ref() != Some(&current) {
                    let state = app.state::<Mutex<History>>();
                    let mut history = state.lock().unwrap();
                    last = Some(current.clone());
                    let evicted = history.add(current);

                    eviction::forget_entries(evicted, &images);

                    if let Err(e) = store.save(&history) {
                        eprintln!("save failed: {e}");
                    }
                }
            }

            thread::sleep(Duration::from_millis(500));
        }
    });
}
