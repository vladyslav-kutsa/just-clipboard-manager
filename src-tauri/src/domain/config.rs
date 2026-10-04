use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Дії всередині вікна, яким можна призначити власну комбінацію.
/// Системні конвенції (↑↓, ⏎, Esc, ⌘1–9) навмисно не налаштовуються.
pub const ACTION_PIN: &str = "pin";
pub const ACTION_PREVIEW: &str = "preview";

fn default_width() -> f64 {
    640.0
}
fn default_height() -> f64 {
    460.0
}
fn default_resizable() -> bool {
    true
}
fn default_locale() -> String {
    String::from("uk")
}
fn default_shortcuts() -> HashMap<String, String> {
    HashMap::from([
        (String::from(ACTION_PIN), String::from("Mod+KeyP")),
        (String::from(ACTION_PREVIEW), String::from("Space")),
    ])
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    hotkey: String,
    recent_limit: usize,
    // нові поля з #[serde(default)]: старий config.json без них читається далі
    #[serde(default = "default_width")]
    window_width: f64,
    #[serde(default = "default_height")]
    window_height: f64,
    #[serde(default = "default_resizable")]
    resizable: bool,
    #[serde(default = "default_locale")]
    locale: String,
    #[serde(default = "default_shortcuts")]
    shortcuts: HashMap<String, String>,
}

impl Config {
    pub fn hotkey(&self) -> &str {
        &self.hotkey
    }

    pub fn set_hotkey(&mut self, combination: String) {
        self.hotkey = combination;
    }

    pub fn recent_limit(&self) -> usize {
        self.recent_limit
    }

    pub fn set_recent_limit(&mut self, limit: usize) {
        self.recent_limit = limit;
    }

    pub fn window_size(&self) -> (f64, f64) {
        (self.window_width, self.window_height)
    }

    pub fn set_window_size(&mut self, width: f64, height: f64) {
        self.window_width = width;
        self.window_height = height;
    }

    pub fn resizable(&self) -> bool {
        self.resizable
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn set_locale(&mut self, locale: String) {
        self.locale = locale;
    }

    pub fn shortcuts(&self) -> &HashMap<String, String> {
        &self.shortcuts
    }

    /// Призначає комбінацію дії. `None` — скинути до дефолтної.
    pub fn set_shortcut(&mut self, action: String, combination: Option<String>) {
        match combination {
            Some(combo) => {
                self.shortcuts.insert(action, combo);
            }
            None => {
                self.shortcuts.remove(&action);
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: String::from("CmdOrCtrl+Shift+V"),
            recent_limit: 10,
            window_width: default_width(),
            window_height: default_height(),
            resizable: default_resizable(),
            locale: default_locale(),
            shortcuts: default_shortcuts(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_both_shortcuts() {
        let c = Config::default();
        assert_eq!(c.shortcuts().get(ACTION_PIN).map(String::as_str), Some("Mod+KeyP"));
        assert_eq!(c.shortcuts().get(ACTION_PREVIEW).map(String::as_str), Some("Space"));
    }

    #[test]
    fn set_shortcut_overwrites() {
        let mut c = Config::default();
        c.set_shortcut(String::from(ACTION_PIN), Some(String::from("Mod+KeyB")));
        assert_eq!(c.shortcuts().get(ACTION_PIN).map(String::as_str), Some("Mod+KeyB"));
    }

    #[test]
    fn set_shortcut_none_resets_to_absent() {
        let mut c = Config::default();
        c.set_shortcut(String::from(ACTION_PIN), None);
        assert!(c.shortcuts().get(ACTION_PIN).is_none());
    }

    #[test]
    fn old_config_without_new_fields_still_loads() {
        // саме цей випадок — наявний config.json на диску
        let old = r#"{"hotkey":"CmdOrCtrl+Shift+V","recent_limit":10}"#;
        let c: Config = serde_json::from_str(old).expect("старий конфіг має читатись");
        assert_eq!(c.window_size(), (640.0, 460.0));
        assert!(c.resizable());
        assert_eq!(c.locale(), "uk");
        assert_eq!(c.shortcuts().len(), 2);
    }
}
