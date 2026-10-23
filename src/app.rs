/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗         ██╗   ███████╗    ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝        ███║   ██╔════╝   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║           ╚██║   ███████╗   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║            ██║   ╚════██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║            ██║██╗███████║██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝            ╚═╝╚═╝╚══════╝╚═╝ ╚═════╝

Made with ♥ by tfaullk

*/



use crate::config::Settings;
use crate::engine::{TestMode, TestStatus, TypingTest};
use crate::themes::{self, Theme, ThemeWarning};
use crate::words::WordGenerator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Test,
    Results,
    ThemePicker,
    Help,
    Update,
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
    pub menu_index: usize,
    pub update_state: crate::update::TuiUpdateState,
    generator: WordGenerator,
}

fn word_count_for(mode: TestMode) -> usize {
    match mode {
        TestMode::Time(_) => 120,
        TestMode::Words(n) => n,
        TestMode::Quote => 30,
    }
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
            screen: Screen::Menu,
            theme,
            themes: available_themes,
            settings,
            test,
            theme_index: 0,
            // if some theme files were broken, let the user know but don't die
            theme_warnings,
            theme_warning_index: 0,
            menu_index: 0,
            update_state: crate::update::TuiUpdateState::default(),
            generator,
        })
    }

    pub const MENU_ITEMS: usize = 5;

    pub fn menu_labels() -> [&'static str; Self::MENU_ITEMS] {
        ["Start", "Themes", "Keybinds", "Update", "Exit"]
    }

    pub fn menu_up(&mut self) {
        if self.menu_index == 0 {
            self.menu_index = Self::MENU_ITEMS - 1;
        } else {
            self.menu_index -= 1;
        }
    }

    pub fn menu_down(&mut self) {
        self.menu_index = (self.menu_index + 1) % Self::MENU_ITEMS;
    }

    pub fn menu_activate(&mut self) {
        match self.menu_index {
            0 => {
                self.restart_test(true);
                self.screen = Screen::Test;
            }
            1 => {
                self.screen = Screen::ThemePicker;
                self.theme_index = self
                    .themes
                    .iter()
                    .position(|theme| theme.name == self.theme.name)
                    .unwrap_or(0);
            }
            2 => {
                self.screen = Screen::Help;
            }
            3 => {
                self.update_state = crate::update::TuiUpdateState::default();
                self.screen = Screen::Update;
            }
            4 => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn restart_test(&mut self, new_words: bool) {
        let count = word_count_for(self.test.mode);
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
        self.generator = WordGenerator::new(next.clone()); // rebuild pool
        self.settings.word_set = next.label();
        self.restart_test(true);
    }

    pub fn word_set_display_name(&self) -> String {
        self.generator.set.display_name()
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

    pub fn check_personal_best(&mut self) {
        let mode_label = self.test.mode.label();
        let current_wpm = self.test.stats.wpm();

        if current_wpm > 0.0 {
            let pb = self.settings.get_pb(&mode_label);
            if current_wpm > pb {
                self.settings.personal_bests.insert(mode_label, current_wpm);
                let _ = self.settings.save();
            }
        }
    }

    pub fn finish_test_and_show_results(&mut self) {
        if self.screen != Screen::Test {
            return;
        }
        self.check_personal_best();
        let _ = self.settings.save();
        self.screen = Screen::Results;
    }

    pub fn tick(&mut self) {
        self.test.tick();
        // catches the "timer ran out" case, since there's no keypress for that
        if self.test.status == TestStatus::Finished && self.screen == Screen::Test {
            self.finish_test_and_show_results();
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
