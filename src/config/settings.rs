/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗         ██╗   ███████╗    ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝        ███║   ██╔════╝   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║           ╚██║   ███████╗   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║            ██║   ╚════██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║            ██║██╗███████║██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝            ╚═╝╚═╝╚══════╝╚═╝ ╚═════╝

Made with ♥ by tfaullk

*/

use std::collections::HashMap;
use super::paths;
use crate::engine::TestMode;
use crate::error::Result;
use crate::words::WordSet;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub word_set: String,
    // stored as a string like "time 30" so the toml stays easy to read
    pub mode: String,
    pub show_live_wpm: bool,
    pub show_accuracy: bool,
    pub cursor_style: CursorStyle,
    pub show_wpm_graph: bool,
    #[serde(default)]
    pub personal_bests: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dracula".into(),
            word_set: "english".into(),
            mode: "time 30".into(),
            show_live_wpm: true,
            show_accuracy: true,
            cursor_style: CursorStyle::Block,
            show_wpm_graph: true,
            personal_bests: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        // if anything goes wrong (missing file, bad toml) we just use defaults
        let path = paths::settings_file();
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        paths::ensure_config_dir()?;
        let path = paths::settings_file();

        let tmp = path.with_extension("toml.tmp");
        let s = toml::to_string_pretty(self)?;
        fs::write(&tmp, s.as_bytes())?;

        #[cfg(windows)]
        {
            let _ = fs::remove_file(&path);
        }

        fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn get_pb(&self, mode: &str) -> f64 {
        *self.personal_bests.get(mode).unwrap_or(&0.0)
    }

    pub fn word_set(&self) -> WordSet {
        WordSet::from_label(&self.word_set).unwrap_or(WordSet::English)
    }

    // "time 30" -> TestMode::Time(30), you get the idea
    pub fn test_mode(&self) -> TestMode {
        parse_mode(&self.mode).unwrap_or(TestMode::Time(30)) 
    }
}

fn parse_mode(s: &str) -> Option<TestMode> {
    let mut parts = s.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("time"), Some(n)) => Some(TestMode::Time(n.parse().ok()?)),
        (Some("words"), Some(n)) => Some(TestMode::Words(n.parse().ok()?)),
        (Some("quote"), _) => Some(TestMode::Quote),
        _ => None,
    }
}
