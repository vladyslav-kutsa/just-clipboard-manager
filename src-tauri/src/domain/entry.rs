use crate::domain::clip_item::Preview;

use super::clip_item::ClipItem;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entry{
    id: Uuid,
    item: ClipItem,
}

impl Entry {
    pub fn new(item: ClipItem) -> Entry {
        Entry { id: Uuid::now_v7(), item }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn item(&self) -> &ClipItem {
        &self.item
    }

    pub fn preview(&self) -> String {
        self.item.preview()
    }
}
