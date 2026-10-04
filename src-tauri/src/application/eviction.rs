use crate::application::ports::SharedImageStore;
use crate::domain::clip_item::ClipItem;
use crate::domain::entry::Entry;

pub fn forget_entries(entries: impl IntoIterator<Item = Entry>, images: &SharedImageStore) {
    for entry in entries {
        if let ClipItem::Image { hash, .. } = entry.item() {
            if let Err(e) = images.delete(hash) {
                eprintln!("clear image failed: {e}");
            }
        }
    }
}
