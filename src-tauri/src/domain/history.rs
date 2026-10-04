use super::clip_item::ClipItem;
use super::entry::Entry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct History {
    pinned: Vec<Entry>,
    recent: Vec<Entry>,
    recent_limit: usize,
}

impl History {
    pub fn new(recent_limit: usize) -> History {
        History {
            pinned: vec![],
            recent: vec![],
            recent_limit: recent_limit,
        }
    }

    pub fn set_recent_limit(&mut self, limit: usize) -> Vec<Entry> {
        let mut entries: Vec<Entry> = Vec::new();
        self.recent_limit = limit;

        while self.recent.len() > self.recent_limit {
            entries.push(self.recent.remove(0));
        }

        entries
    }

    pub fn recent(&self) -> &[Entry] {
        &self.recent
    }

    pub fn pinned(&self) -> &[Entry] {
        &self.pinned
    }

    pub fn add(&mut self, item: ClipItem) -> Option<Entry> {
        if self.pinned.iter().any(|i| i.item() == &item) {
            return None;
        }

        if let Some(position) = self.recent.iter().position(|i| i.item() == &item) {
            self.recent.remove(position);
        }

        self.recent.push(Entry::new(item));

        if self.recent.len() > self.recent_limit {
            Some(self.recent.remove(0))
        } else {
            None
        }
    }

    pub fn pin(&mut self, id: Uuid) {
        if let Some(position) = self.recent.iter().position(|i| i.id() == id) {
            self.pinned.push(self.recent.remove(position));
        }
    }

    pub fn remove_pinned(&mut self, id: Uuid) -> Option<Entry> {
        self.pinned
            .iter()
            .position(|i| i.id() == id)
            .map(|p| self.pinned.remove(p))
    }

    pub fn len(&self) -> usize {
        self.recent.len()
    }

    pub fn pinned_len(&self) -> usize {
        self.pinned.len()
    }

    pub fn text_count(&self) -> usize {
        self.recent
            .iter()
            .filter(|i| matches!(i.item(), ClipItem::Text(_)))
            .count()
    }

    pub fn previews(&self) -> Vec<String> {
        self.recent.iter().map(|i| i.preview()).collect()
    }

    pub fn pinned_previews(&self) -> Vec<String> {
        self.pinned.iter().map(|i| i.preview()).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Entry> {
        let query_lowercase = query.to_lowercase();
        self.pinned.iter()
            .chain(self.recent.iter())
            .filter(|i| matches!(i.item(), ClipItem::Text(s) if s.to_lowercase().contains(&query_lowercase)))
            .collect()
    }

    pub fn image_hashes(&self) -> Vec<String> {
        self.pinned
            .iter()
            .chain(self.recent.iter())
            .filter_map(|i| {
                if let ClipItem::Image { hash, .. } = i.item() {
                    Some(hash.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // хелпер: коротко зробити текстовий елемент
    fn t(s: &str) -> ClipItem {
        ClipItem::Text(String::from(s))
    }
    fn img(hash: &str) -> ClipItem {
        ClipItem::Image {
            hash: String::from(hash),
            width: 1,
            height: 1,
        }
    }

    #[test]
    fn new_history_is_empty() {
        let h = History::new(10);
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn history_add_item_len() {
        let mut h = History::new(10);
        h.add(t("a"));
        let before = h.len();
        h.add(t("b"));
        assert_eq!(h.len(), before + 1);
    }

    #[test]
    fn history_add_move_to_front() {
        let mut h = History::new(10);
        h.add(t("1"));
        h.add(t("2"));
        h.add(t("3"));
        h.add(t("4"));
        h.add(t("1"));
        assert_eq!(h.previews(), vec!["2", "3", "4", "1"]);
    }

    #[test]
    fn history_set_recent_limit() {
        let mut h = History::new(5);
        h.add(t("1"));
        h.add(t("2"));
        h.add(t("3"));
        let will_be_remove: Vec<String> = h.recent.iter().map(|e| e.preview()).collect();
        h.add(t("4"));
        h.add(t("5"));
        let removed: Vec<String> = h.set_recent_limit(2).iter().map(|e| e.preview()).collect();
        assert_eq!(h.previews(), vec!["4", "5"]);
        assert_eq!(removed, will_be_remove);
    }

    #[test]
    fn history_text_count() {
        let mut h = History::new(10);
        h.add(t("a"));
        h.add(t("b"));
        h.add(ClipItem::Image {
            hash: String::new(),
            width: 180,
            height: 120,
        });
        assert_eq!(h.text_count(), 2);
    }

    #[test]
    fn history_previews() {
        let mut h = History::new(10);
        h.add(t("a"));
        h.add(t("b"));
        h.add(ClipItem::Image {
            hash: String::new(),
            width: 10,
            height: 10,
        });
        assert_eq!(h.previews(), vec!["a", "b", "image 10x10"]);
    }

    #[test]
    fn history_limit() {
        let mut h = History::new(2);
        h.add(t("1"));
        h.add(t("2"));
        let evicted = h.add(t("3"));
        assert_eq!(h.previews(), vec!["2", "3"]);
        assert_eq!(evicted.unwrap().preview(), "1");
    }

    #[test]
    fn history_pin() {
        let mut h = History::new(10);
        for s in ["1", "2", "3", "4", "5"] {
            h.add(t(s));
        }
        h.pin(h.recent()[0].id()); // забирає найстаріший ("1") з recent у pinned
        assert_eq!(h.pinned_previews(), vec!["1"]);
    }

    #[test]
    fn history_remove_pinned() {
        let mut h = History::new(10);
        h.add(t("1"));
        h.add(t("2"));
        h.pin(h.recent()[0].id()); // pinned: ["1"]
        h.pin(h.recent()[0].id()); // pinned: ["1","2"]
        let unpinned_id = h.pinned()[0].id();
        let unpinned = h.remove_pinned(h.pinned()[0].id());
        assert_eq!(h.pinned_previews(), vec!["2"]);
        assert_eq!(unpinned.unwrap().id(), unpinned_id);
    }

    #[test]
    fn history_search() {
        let mut h = History::new(10);
        h.add(t("001 QwErTy"));
        h.add(t("111 TeSt"));
        h.pin(h.recent()[0].id());
        h.pin(h.recent()[0].id());
        h.add(t("002 qWeRtY"));
        h.add(t("222 tEsT"));
        assert_eq!(h.search("qwerty").len(), 2);
    }

    #[test]
    fn history_add_skips_pinned_duplicate() {
        let mut h = History::new(10);
        h.add(t("foo"));
        h.pin(h.recent()[0].id());
        let evicted = h.add(t("foo"));
        assert_eq!(h.recent().len(), 0);
        assert!(evicted.is_none());
    }

    #[test]
    fn add_without_overflow_returns_none() {
        let mut h = History::new(5);
        h.add(t("1"));
        let evicted = h.add(t("2"));
        assert!(evicted.is_none());
    }

    #[test]
    fn add_duplicate_returns_none() {
        let mut h = History::new(2);
        h.add(t("a"));
        h.add(t("b"));
        let evicted = h.add(t("a"));
        assert!(evicted.is_none());
        assert_eq!(h.previews(), vec!["b", "a"]);
    }

    #[test]
    fn remove_pinned_missing_id_returns_none() {
        let mut h = History::new(5);
        let evicted = h.remove_pinned(Uuid::now_v7());
        assert!(evicted.is_none());
    }

    #[test]
    fn image_hashes() {
        let mut h = History::new(5);
        h.add(t("t1"));
        h.add(img("i1"));
        h.add(img("i2"));
        h.pin(h.recent()[1].id());
        let hashes = h.image_hashes();

        assert_eq!(hashes, vec!["i1", "i2"]);
    }
}
