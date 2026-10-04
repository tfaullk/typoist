/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░░░███╗░░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░░████║░░░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░██╔██║░░░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░╚═╝██║░░░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗███████╗██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚══════╝╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use super::paths;
use crate::engine::TestMode;
use crate::error::Result;
use crate::words::WordSet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub word_set: String,
    // stored as a string like "time 30" so the toml stays easy to read
    pub mode: String,
    pub show_live_wpm: bool,
    pub show_accuracy: bool,
    pub cursor_style: CursorStyle,
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
        let dir = paths::config_dir();
        fs::create_dir_all(&dir)?;
        let path = paths::settings_file();
        let mut file = fs::File::create(&path)?;
        let s = toml::to_string_pretty(self)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    pub fn word_set(&self) -> WordSet {
        match self.word_set.as_str() {
            "code" => WordSet::Code,
            "EnglishPunctuation" => WordSet::EnglishPunctuation,
            "code+symbols" => WordSet::CodeSymbols,
            _ => WordSet::English,
        }
    }

    // "time 30" -> TestMode::Time(30), you get the idea
    pub fn test_mode(&self) -> TestMode {
        let mut parts = self.mode.split_whitespace();
        match (parts.next(), parts.next()) {
            (Some("time"), Some(n)) => TestMode::Time(n.parse().unwrap_or(30)),
            (Some("words"), Some(n)) => TestMode::Words(n.parse().unwrap_or(25)),
            (Some("quote"), _) => TestMode::Quote,
            _ => TestMode::Time(30),
        }
    }
}
