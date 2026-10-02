/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗     ██╗    ██████╗     ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝    ███║   ██╔═████╗   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║       ╚██║   ██║██╔██║   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║        ██║   ████╔╝██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║        ██║██╗╚██████╔╝██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝        ╚═╝╚═╝ ╚═════╝ ╚═╝ ╚═════╝

Made with ♥ by tfaullk


*/

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

// colors are stored as [r, g, b] in the toml so themes are easy to hand-edit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub correct: [u8; 3],
    pub incorrect: [u8; 3],
    pub pending: [u8; 3],
    pub cursor: [u8; 3],
    pub accent: [u8; 3],
    pub sub: [u8; 3],
}

// small helpers so the rest of the code doesn't hand-roll Color::Rgb everywhere
impl Theme {
    pub fn bg(&self) -> Color { Color::Rgb(self.background[0], self.background[1], self.background[2]) }
    
    pub fn fg(&self) -> Color { Color::Rgb(self.foreground[0], self.foreground[1], self.foreground[2]) }

    pub fn correct(&self) -> Color { Color::Rgb(self.correct[0], self.correct[1], self.correct[2]) }

    pub fn incorrect(&self) -> Color { Color::Rgb(self.incorrect[0], self.incorrect[1], self.incorrect[2]) }

    pub fn pending(&self) -> Color { Color::Rgb(self.pending[0], self.pending[1], self.pending[2]) }

    pub fn cursor(&self) -> Color { Color::Rgb(self.cursor[0], self.cursor[1], self.cursor[2]) }

    pub fn accent(&self) -> Color { Color::Rgb(self.accent[0], self.accent[1], self.accent[2]) }

    pub fn sub(&self) -> Color { Color::Rgb(self.sub[0], self.sub[1], self.sub[2]) }

    pub fn validate(&self) -> crate::error::Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::error::TypoistError::ThemeEmptyName(
                self.name.clone(),
            ));
        }

        // no path tricks — the name becomes a filename when importing
        if self.name.contains('/') || self.name.contains('\\') {
            return Err(crate::error::TypoistError::ThemeInvalidName {
                name: self.name.clone(),
                reason: "theme names cannot contain path separators".into(),
            });
        }

        if self.name.contains('\0') {
            return Err(crate::error::TypoistError::ThemeInvalidName {
                name: self.name.clone(),
                reason: "theme names cannot contain null characters".into(),
            });
        }
        Ok(())
    }
}
