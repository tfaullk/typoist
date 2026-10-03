/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗     ██╗    ██████╗     ██████╗
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝    ███║   ██╔═████╗   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║       ╚██║   ██║██╔██║   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║        ██║   ████╔╝██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║        ██║██╗╚██████╔╝██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝        ╚═╝╚═╝ ╚═════╝ ╚═╝ ╚═════╝

Made with ♥ by tfaullk


*/

use super::stats::Stats;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMode {
    Time(u64),
    Words(usize),
    Quote,
}

impl TestMode {
    pub fn label(&self) -> String {
        match self {
            TestMode::Time(s) => format!("time {}", s),
            TestMode::Words(n) => format!("words {}", n),
            TestMode::Quote => "quote".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    #[allow(dead_code)]
    Pending,
    Correct,
    Incorrect,
    Extra,
}

#[derive(Debug, Clone)]
pub struct TypedChar {
    pub actual: char,
    pub state: CharState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    NotStarted,
    Running,
    Finished,
}

pub struct TypingTest {
    pub words: Vec<String>,
    pub typed: Vec<TypedChar>,
    pub cursor: usize,
    pub status: TestStatus,
    pub mode: TestMode,
    pub stats: Stats,
    started_at: Option<Instant>,
    last_sample_at: Option<Instant>,
    sample_keystrokes: usize,
}

impl TypingTest {
    pub fn new(words: Vec<String>, mode: TestMode) -> Self {
        // one slot per char plus the spaces between words
        let total_chars: usize =
            words.iter().map(|w| w.len()).sum::<usize>() + words.len().saturating_sub(1);
        Self {
            words,
            typed: Vec::with_capacity(total_chars),
            cursor: 0,
            status: TestStatus::NotStarted,
            mode,
            stats: Stats::new(),
            started_at: None,
            last_sample_at: None,
            sample_keystrokes: 0,
        }
    }

    // flatten everything into one stream of chars, spaces included
    pub fn target_chars(&self) -> Vec<char> {
        let mut out = Vec::new();
        for (i, w) in self.words.iter().enumerate() {
            for c in w.chars() {
                out.push(c);
            }
            if i != self.words.len() - 1 {
                out.push(' ');
            }
        }
        out
    }

    // the timer only starts on the first keystroke, like monkeytype
    pub fn start(&mut self) {
        if self.status == TestStatus::NotStarted {
            let now = Instant::now();
            self.started_at = Some(now);
            self.last_sample_at = Some(now);
            self.status = TestStatus::Running;
        }
    }

    pub fn push_char(&mut self, c: char) {
        if self.status == TestStatus::Finished {
            return;
        }
        self.start();

        let target = self.target_chars();
        let expected = target.get(self.cursor).copied();

        // typing past the end counts as extra
        let state = match expected {
            Some(e) if e == c => CharState::Correct,
            Some(_) => CharState::Incorrect,
            None => CharState::Extra,
        };

        self.typed.push(TypedChar { actual: c, state });
        self.cursor += 1;

        self.stats.total_keystrokes += 1;
        match state {
            CharState::Correct => self.stats.correct_keystrokes += 1,
            CharState::Incorrect | CharState::Extra => self.stats.incorrect_keystrokes += 1,
            CharState::Pending => {}
        }
        self.sample_if_due();

        if self.cursor >= target.len() && self.all_words_complete(&target) {
            self.finish();
        }
    }

    fn all_words_complete(&self, target: &[char]) -> bool {
        self.cursor >= target.len()
    }

    // note: this doesn't undo the mistake in the stats, it's display-only
    pub fn backspace(&mut self) {
        if self.status != TestStatus::Running || self.typed.is_empty() {
            return;
        }
        self.typed.pop();
        self.cursor = self.cursor.saturating_sub(1);
    }

    // called from the main loop's tick, mostly to keep elapsed time fresh
    // and end timed tests
    pub fn tick(&mut self) {
        if self.status != TestStatus::Running {
            return;
        }
        let Some(started) = self.started_at else {
            return;
        };
        self.stats.elapsed = started.elapsed();
        self.sample_if_due();

        if let TestMode::Time(secs) = self.mode {
            if self.stats.elapsed.as_secs() >= secs {
                self.finish();
            }
        }
    }

    // grab a wpm sample once a second, that's what consistency is built from
    fn sample_if_due(&mut self) {
        let Some(started) = self.started_at else {
            return;
        };
        let now = Instant::now();
        self.stats.elapsed = now.duration_since(started);
        let Some(last) = self.last_sample_at else {
            return;
        };
        if now.duration_since(last) >= Duration::from_secs(1) {
            let wpm = self.stats.wpm();
            self.stats.push_wpm_sample(wpm);
            self.last_sample_at = Some(now);
            self.sample_keystrokes = 0;
        }
    }

    pub fn finish(&mut self) {
        if self.status == TestStatus::Finished {
            return;
        }
        if let Some(started) = self.started_at {
            self.stats.elapsed = started.elapsed();
        }
        self.status = TestStatus::Finished;
    }

    pub fn reset(&mut self, new_words: Option<Vec<String>>) {
        if let Some(w) = new_words {
            self.words = w;
        }
        // keep the old capacity around, saves a realloc
        let cap = self.typed.capacity();
        self.typed = Vec::with_capacity(cap);
        self.cursor = 0;
        self.status = TestStatus::NotStarted;
        self.stats = Stats::new();
        self.started_at = None;
        self.last_sample_at = None;
    }
}
