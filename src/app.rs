/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use crate::verify::{Identity, PbChain, RunRecord};
use crate::config::Settings;
use crate::engine::{StenoStatus, StenoTest, TestMode, TestStatus, TypingTest};
use crate::input::keybinds::KeyAction;
use crate::themes::{self, Theme, ThemeWarning};
use crate::words::WordGenerator;

fn q6(x: f64) -> f64 {
    (x * 1_000_000.0).round() / 1_000_000.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Test,
    Steno,
    Results,
    ThemePicker,
    Help,
    Keybinds,
    Update,
}

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    pub theme: Theme,
    pub themes: Vec<Theme>,
    pub settings: Settings,
    pub test: TypingTest,
    pub steno: StenoTest,
    pub theme_index: usize,
    pub theme_warnings: Vec<ThemeWarning>,
    pub theme_warning_index: usize,
    pub menu_index: usize,
    pub update_state: crate::update::TuiUpdateState,
    pub keybind_index: usize,
    pub keybind_capture: Option<KeyAction>,
    pub keybind_message: Option<String>,
    pub help_screen: Screen,
    generator: WordGenerator,
    identity: Identity,
    chain: PbChain,
}

fn word_count_for(mode: TestMode) -> usize {
    match mode {
        TestMode::Time(_) => 120,
        TestMode::Words(n) => n,
        TestMode::Quote => 30,
    }
}

fn build_steno_prompts(settings: &Settings) -> Vec<crate::engine::StenoPrompt> {
    use rand::seq::SliceRandom;
    let mut pool = crate::engine::steno::load_custom_prompts()
        .unwrap_or_else(crate::engine::steno::builtin_prompts);

    let mut rng = rand::thread_rng();
    pool.shuffle(&mut rng);

    let count = settings.steno_prompt_count.max(1).min(pool.len());
    pool.truncate(count);
    pool
}

impl App {
    pub fn new(settings: Settings) -> crate::error::Result<Self> {
        let (available_themes, theme_warnings) = themes::all()?;
        let identity = Identity::load_or_create()?;
        let chain = PbChain::open();

        let theme = available_themes
            .iter()
            .find(|theme| theme.name == settings.theme)
            .cloned()
            .or_else(|| available_themes.first().cloned())
            .ok_or(crate::error::TypoistError::NoThemesAvailable)?;

        let word_set = settings.word_set();
        let mode = settings.test_mode();
        let mut generator = WordGenerator::new(word_set);
        let count = word_count_for(mode);
        let words = generator.generate(count);
        let test = TypingTest::new(words, mode);

        let steno_prompts = build_steno_prompts(&settings);
        let steno = StenoTest::new(steno_prompts, settings.steno_chord_timeout_ms);

        Ok(Self {
            should_quit: false,
            screen: Screen::Menu,
            theme,
            themes: available_themes,
            settings,
            test,
            steno,
            theme_index: 0,
            theme_warnings,
            theme_warning_index: 0,
            menu_index: 0,
            update_state: crate::update::TuiUpdateState::default(),
            keybind_index: 0,
            keybind_capture: None,
            keybind_message: None,
            help_screen: Screen::Menu,
            generator,
            identity,
            chain,
        })
    }

    pub const MENU_ITEMS: usize = 6;

    pub fn menu_labels() -> [&'static str; Self::MENU_ITEMS] {
        ["Start", "Steno", "Themes", "Keybinds", "Update", "Exit"]
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
                self.restart_steno(true);
                self.screen = Screen::Steno;
            }
            2 => {
                self.screen = Screen::ThemePicker;
                self.theme_index = self
                    .themes
                    .iter()
                    .position(|theme| theme.name == self.theme.name)
                    .unwrap_or(0);
            }
            3 => {
                self.help_screen = Screen::Menu;
                self.keybind_index = 0;
                self.keybind_capture = None;
                self.keybind_message = None;
                self.screen = Screen::Keybinds;
            }
            4 => {
                self.update_state = crate::update::TuiUpdateState::default();
                self.screen = Screen::Update;
            }
            5 => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn restart_test(&mut self, new_words: bool) {
        let count = word_count_for(self.test.mode);
        let new = if new_words {
            Some(self.generator.generate(count))
        } else {
            None
        };
        self.test.reset(new);
        self.screen = Screen::Test;
    }

    pub fn restart_steno(&mut self, new_prompts: bool) {
        let prompts = if new_prompts {
            Some(build_steno_prompts(&self.settings))
        } else {
            None
        };
        self.steno.reset(prompts);
        self.screen = Screen::Steno;
    }

    pub fn cycle_word_set(&mut self) {
        let next = self.generator.set.next();
        self.generator = WordGenerator::new(next.clone());
        self.settings.word_set = next.label();
        self.restart_test(true);
    }

    pub fn word_set_display_name(&self) -> String {
        self.generator.set.display_name()
    }

    pub fn cycle_mode(&mut self) {
        use TestMode::*;
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
        if let Some(theme) = self.themes.iter().find(|theme| theme.name == name) {
            self.theme = theme.clone();
        }
    }

    pub fn check_personal_best(&mut self) {
        let stats = &self.test.stats;
        let mode_label = self.test.mode.label();
        let wpm = stats.wpm();
        if wpm <= 0.0 {
            return;
        }

        let target: String = self.test.target_chars().iter().collect();
        let typed: String = self.test.typed.iter().map(|t| t.actual).collect();

        let record = RunRecord {
            schema: crate::verify::record::SCHEMA,
            version: env!("CARGO_PKG_VERSION").to_string(),
            mode: mode_label.clone(),
            word_set: self.settings.word_set.clone(),
            target,
            typed,
            elapsed_ms: stats.elapsed.as_millis() as u64,
            wpm: q6(wpm),
            raw_wpm: q6(stats.raw_wpm()),
            accuracy: q6(stats.accuracy()),
            consistency: q6(stats.consistency()),
            wpm_samples: stats.wpm_samples.iter().map(|&s| q6(s)).collect(),
            started_at_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            nonce: {
                use rand::RngCore;
                let mut b = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut b);
                hex::encode(b)
            },
            prev_hash: String::new(),
            sig: String::new(),
            kind: String::new(),
            strokes: Vec::new(),
        };

        if let crate::verify::audit::AuditResult::Rejected(errs) =
            crate::verify::audit::audit(&record)
        {
            eprintln!("typoist: run failed self-audit, not recording: {:?}", errs);
            return;
        }

        if let Err(e) = self.chain.append(record, &self.identity) {
            eprintln!("typoist: could not append to pb chain: {}", e);
        }

        let pb = self.settings.get_pb(&mode_label);
        if wpm > pb {
            self.settings.personal_bests.insert(mode_label, wpm);
            let _ = self.settings.save();
        }
    }

    pub fn check_steno_personal_best(&mut self) {
        let wpm = self.steno.wpm();
        if wpm <= 0.0 || self.steno.strokes.is_empty() {
            return;
        }

        let mut prev = std::time::Duration::ZERO;
        let strokes: Vec<crate::verify::StenoStrokeRecord> = self
            .steno
            .strokes
            .iter()
            .map(|s| {
                let delta = s.elapsed.saturating_sub(prev);
                prev = s.elapsed;
                crate::verify::StenoStrokeRecord {
                    expected: s.expected.clone(),
                    actual: s.actual.clone(),
                    correct: s.correct,
                    elapsed_ms: delta.as_millis() as u64,
                }
            })
            .collect();

        let target: String = strokes
            .iter()
            .map(|s| s.expected.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let typed: String = strokes
            .iter()
            .map(|s| s.expected.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let elapsed_secs = self.steno.stats.elapsed.as_secs_f64().max(1e-9);
        let raw_wpm = (self.steno.stroke_count as f64 / 5.0) / (elapsed_secs / 60.0);

        let record = RunRecord {
            schema: crate::verify::record::SCHEMA,
            version: env!("CARGO_PKG_VERSION").to_string(),
            mode: format!("steno {}", self.steno.total_prompts()),
            word_set: "steno".to_string(),
            target,
            typed,
            elapsed_ms: self.steno.stats.elapsed.as_millis() as u64,
            wpm: q6(wpm),
            raw_wpm: q6(raw_wpm),
            accuracy: q6(self.steno.accuracy()),
            consistency: 0.0,
            wpm_samples: Vec::new(),
            started_at_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            nonce: {
                use rand::RngCore;
                let mut b = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut b);
                hex::encode(b)
            },
            prev_hash: String::new(),
            sig: String::new(),
            kind: "steno".to_string(),
            strokes,
        };

        if let crate::verify::audit::AuditResult::Rejected(errs) =
            crate::verify::audit::audit(&record)
        {
            eprintln!("typoist: steno run failed self-audit, not recording: {:?}", errs);
            return;
        }

        if let Err(e) = self.chain.append(record, &self.identity) {
            eprintln!("typoist: could not append steno to pb chain: {}", e);
        }

        let key = "steno".to_string();
        let pb = self.settings.get_pb(&key);
        if wpm > pb {
            self.settings.personal_bests.insert(key, wpm);
            let _ = self.settings.save();
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

    pub fn finish_steno_and_show_results(&mut self) {
        if self.screen != Screen::Steno {
            return;
        }
        self.check_steno_personal_best();
        let _ = self.settings.save();
        self.screen = Screen::Results;
    }

    pub fn tick(&mut self) {
        self.test.tick();
        if self.test.status == TestStatus::Finished && self.screen == Screen::Test {
            self.finish_test_and_show_results();
        }

        self.steno.tick();
        if self.steno.status == StenoStatus::Finished && self.screen == Screen::Steno {
            self.finish_steno_and_show_results();
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
