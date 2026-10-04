use crate::application::ports::HistoryStore;
use crate::domain::history::History;
use std::error::Error;
use std::path::PathBuf;

pub struct JsonStore {
    path: PathBuf
}

impl JsonStore {
    pub fn new(path: PathBuf) -> JsonStore {
        JsonStore { path: path}
    }
}

impl HistoryStore for JsonStore {
    fn save(&self, history: &History) -> Result<(), Box<dyn Error>>{
        let row = serde_json::to_string(history)?;
        std::fs::write(&self.path, row)?;
        Ok(())
    }

    fn load(&self) -> Result<History, Box<dyn Error>>{
        let text = std::fs::read_to_string(&self.path)?;
        let history: History = serde_json::from_str(&text)?;
        Ok(history)
    }
}
