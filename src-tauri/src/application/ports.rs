use crate::domain::config::Config;
use crate::domain::history::History;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

pub type SharedStore = Arc<dyn HistoryStore + Send + Sync>;
pub trait HistoryStore {
    fn save(&self, history: &History) -> Result<(), Box<dyn Error>>;
    fn load(&self) -> Result<History, Box<dyn Error>>;
}

pub type SharedImageStore = Arc<dyn ImageStore + Send + Sync>;
pub trait ImageStore {
    /// Зберігає картинку, повертає її хеш. Ідемпотентний:
    /// якщо файл із таким хешем уже є — нічого не переписує.
    fn save(&self, rgba: &[u8], width: u32, height: u32) -> Result<String, Box<dyn Error>>;
    fn load_rgba(&self, hash: &str) -> Result<(Vec<u8>, u32, u32), Box<dyn Error>>;
    fn original_path(&self, hash: &str) -> PathBuf;
    fn thumb_path(&self, hash: &str) -> PathBuf;
    fn delete(&self, hash: &str) -> Result<(), Box<dyn Error>>;
    /// Видаляє все, чого немає в списку — прибирання «сиріт».
    fn cleanup(&self, keep: &[String]) -> Result<(), Box<dyn Error>>;
}

pub type SharedConfigStore = Arc<dyn ConfigStore + Send + Sync>;
pub trait ConfigStore {
    fn save(&self, config: &Config) -> Result<(), Box<dyn Error>>;
    fn load(&self) -> Result<Config, Box<dyn Error>>;
}
