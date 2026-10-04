use crate::application::ports::ConfigStore;
use crate::domain::config::Config;
use std::error::Error;
use std::path::PathBuf;

pub struct JsonConfigStore {
    path: PathBuf
}

impl JsonConfigStore {
    pub fn new(path: PathBuf) -> JsonConfigStore {
        JsonConfigStore { path: path}
    }
}

impl ConfigStore for JsonConfigStore {
    fn save(&self, config: &Config) -> Result<(), Box<dyn Error>>{
        let row = serde_json::to_string(config)?;
        std::fs::write(&self.path, row)?;
        Ok(())
    }

    fn load(&self) -> Result<Config, Box<dyn Error>>{
        let text = std::fs::read_to_string(&self.path)?;
        let config: Config = serde_json::from_str(&text)?;
        Ok(config)
    }
}
