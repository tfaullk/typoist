/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░░░███╗░░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░░████║░░░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░██╔██║░░░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░╚═╝██║░░░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗███████╗██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚══════╝╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use crate::config::Settings;
use crate::engine::{TestMode, TestStatus, TypingTest};
use crate::themes::{self, Theme, ThemeWarning};
use crate::words::WordGenerator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Test,
    Results,
    ThemePicker,
    Help,
}

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    pub theme: Theme,
    pub themes: Vec<Theme>,
    pub settings: Settings,
    pub test: TypingTest,
    pub theme_index: usize,
    pub theme_warnings: Vec<ThemeWarning>,
    pub theme_warning_index: usize,
    generator: WordGenerator,
}

impl App {
    pub fn new(settings: Settings) -> crate::error::Result<Self> {
        let (available_themes, theme_warnings) = themes::all()?;

        // fall back to the first theme if the one in settings doesn't exist
        // (renamed, deleted, whatever)
        let theme = available_themes
            .iter()
            .find(|theme| theme.name == settings.theme)
            .cloned()
            .or_else(|| available_themes.first().cloned())
            .ok_or(crate::error::TypoistError::NoThemesAvailable)?;

        let word_set = settings.word_set();
        let mode = settings.test_mode();
        let mut generator = WordGenerator::new(word_set);
        // timed tests just need "enough" words, the timer ends them anyway
        let count = match mode {
            TestMode::Time(_) => 120,
            TestMode::Words(n) => n,
            TestMode::Quote => 30,
        };
        let words = generator.generate(count);
        let test = TypingTest::new(words, mode);

        Ok(Self {
            should_quit: false,
            screen: Screen::Test,
            theme,
            themes: available_themes,
            settings,
            test,
            theme_index: 0,
            // if some theme files were broken, let the user know but don't die
            theme_warnings,
            theme_warning_index: 0,
            generator,
        })
    }

    pub fn restart_test(&mut self, new_words: bool) {
        let count = match self.test.mode {
            TestMode::Time(_) => 120,
            TestMode::Words(n) => n,
            TestMode::Quote => 30,
        };
        // esc = same words again, tab = fresh ones
        let new = if new_words {
            Some(self.generator.generate(count))
        } else {
            None
        };
        self.test.reset(new);
        self.screen = Screen::Test;
    }

    pub fn cycle_word_set(&mut self) {
        let next = self.generator.set.next();
        self.generator.set = next;
        self.settings.word_set = next.label().to_string();
        self.restart_test(true);
    }

    pub fn cycle_mode(&mut self) {
        use TestMode::*;
        // walk through the usual presets rather than jumping around randomly
        let next = match self.test.mode {
            Time(15) => Time(30),
            Time(30) => Time(60),
            Time(60) => Time(120),
            Time(_) => Time(15),
            Words(10) => Words(25),
            Words(25) => Words(50),
            Words(50) => Words(100),
            Words(_) => Words(10),
            Quote => Time(30),
        };
        self.test.mode = next;
        self.settings.mode = next.label();
        self.restart_test(true);
    }

    pub fn preview_theme(&mut self, name: &str) {
        // just swaps the live theme, doesn't save until enter is pressed
        if let Some(theme) = self.themes.iter().find(|theme| theme.name == name) {
            self.theme = theme.clone();
        }
    }

    pub fn tick(&mut self) {
        self.test.tick();
        // catches the "timer ran out" case, since there's no keypress for that
        if self.test.status == TestStatus::Finished && self.screen == Screen::Test {
            self.screen = Screen::Results;
            let _ = self.settings.save();
        }
    }

    pub fn current_theme_warning(&self) -> Option<&ThemeWarning> {
        self.theme_warnings.get(self.theme_warning_index)
    }

    pub fn dismiss_theme_warning(&mut self) {
        if self.theme_warnings.is_empty() {
            return;
        }

        self.theme_warning_index += 1;

        if self.theme_warning_index >= self.theme_warnings.len() {
            self.theme_warnings.clear();
            self.theme_warning_index = 0;
        }
    }
}
