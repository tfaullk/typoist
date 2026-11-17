/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use std::time::{Duration, Instant};

use super::stats::Stats;

#[derive(Debug, Clone)]
pub struct StenoPrompt {
    pub word: String,
    pub outline: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StenoStatus {
    NotStarted,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub struct StenoStroke {
    pub expected: String,
    pub actual: String,
    pub correct: bool,
    pub elapsed: Duration,
}

pub struct StenoTest {
    pub prompts: Vec<StenoPrompt>,
    pub index: usize,
    pub status: StenoStatus,
    pub strokes: Vec<StenoStroke>,
    pub pending_keys: Vec<char>,
    pub chord_deadline: Option<Instant>,
    pub chord_timeout: Duration,
    pub stats: Stats,
    pub stroke_count: usize,
    pub correct_strokes: usize,
    started_at: Option<Instant>,
}

impl StenoTest {
    pub fn new(prompts: Vec<StenoPrompt>, chord_timeout_ms: u64) -> Self {
        Self {
            prompts,
            index: 0,
            status: StenoStatus::NotStarted,
            strokes: Vec::new(),
            pending_keys: Vec::new(),
            chord_deadline: None,
            chord_timeout: Duration::from_millis(chord_timeout_ms.max(30)),
            stats: Stats::new(),
            stroke_count: 0,
            correct_strokes: 0,
            started_at: None,
        }
    }

    pub fn current(&self) -> Option<&StenoPrompt> {
        self.prompts.get(self.index)
    }

    pub fn total_prompts(&self) -> usize {
        self.prompts.len()
    }

    pub fn start(&mut self) {
        if self.status == StenoStatus::NotStarted {
            let now = Instant::now();
            self.started_at = Some(now);
            self.status = StenoStatus::Running;
        }
    }

    pub fn push_key(&mut self, c: char) {
        if self.status == StenoStatus::Finished {
            return;
        }
        self.start();
        if self.pending_keys.is_empty() {
            self.chord_deadline = Some(Instant::now() + self.chord_timeout);
        }
        self.pending_keys.push(c);
    }

    pub fn commit_chord(&mut self) {
        if self.pending_keys.is_empty() {
            return;
        }
        if self.status == StenoStatus::Finished {
            self.pending_keys.clear();
            self.chord_deadline = None;
            return;
        }

        let actual: String = self.pending_keys.iter().collect();
        self.pending_keys.clear();
        self.chord_deadline = None;

        let Some(prompt) = self.current().cloned() else {
            return;
        };

        let correct = normalize(&actual) == normalize(&prompt.outline);
        let elapsed = self
            .started_at
            .map(|s| s.elapsed())
            .unwrap_or_default();

        self.strokes.push(StenoStroke {
            expected: prompt.outline.clone(),
            actual: actual.clone(),
            correct,
            elapsed,
        });

        self.stroke_count += 1;
        self.stats.total_keystrokes += 1;

        if correct {
            self.correct_strokes += 1;
            self.stats.correct_keystrokes += 1;
            self.index += 1;
            if self.index >= self.prompts.len() {
                self.finish();
            } else {
                self.stats.incorrect_keystrokes += 1;
            }
        }

        if let Some(started) = self.started_at {
            self.stats.elapsed = started.elapsed();
        }
    }

    pub fn tick(&mut self) {
        if self.status != StenoStatus::Running {
            return;
        }
        let Some(started) = self.started_at else {
            return;
        };
        self.stats.elapsed = started.elapsed();

        if let Some(deadline) = self.chord_deadline {
            if Instant::now() >= deadline && !self.pending_keys.is_empty() {
                self.commit_chord();
            }
        }
    }

    pub fn finish(&mut self) {
        if self.status == StenoStatus::Finished {
            return;
        }
        if let Some(started) = self.started_at {
            self.stats.elapsed = started.elapsed();
        }
        self.status = StenoStatus::Finished;
    }

    pub fn accuracy(&self) -> f64 {
        if self.stroke_count == 0 {
            return 100.0;
        }
        self.correct_strokes as f64 / self.stroke_count as f64 * 100.0
    }

    pub fn wpm(&self) -> f64 {
        let minutes = self.stats.elapsed.as_secs_f64() / 60.0;
        if minutes <= 0.0 {
            return 0.0;
        }
        (self.correct_strokes as f64 / 5.0) / minutes
    }

    pub fn last_strokes(&self, n: usize) -> &[StenoStroke] {
        let start = self.strokes.len().saturating_sub(n);
        &self.strokes[start..]
    }

    pub fn wrong_strokes(&self) -> impl Iterator<Item = &StenoStroke> {
        self.strokes.iter().filter(|s| !s.correct)
    }

    pub fn reset(&mut self, prompts: Option<Vec<StenoPrompt>>) {
        if let Some(p) = prompts {
            self.prompts = p;
        }
        self.index = 0;
        self.status = StenoStatus::NotStarted;
        self.strokes.clear();
        self.pending_keys.clear();
        self.chord_deadline = None;
        self.stats = Stats::new();
        self.stroke_count = 0;
        self.correct_strokes = 0;
        self.started_at = None;
    }
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub fn builtin_prompts() -> Vec<StenoPrompt> {
    const DICT: &[(&str, &str)] = &[
        ("the", "THE"),
        ("and", "APBD"),
        ("you", "U"),
        ("that", "THA"),
        ("was", "WAS"),
        ("for", "TPOR"),
        ("are", "R"),
        ("with", "W"),
        ("his", "HEUS"),
        ("they", "THEU"),
        ("this", "TH"),
        ("have", "HAF"),
        ("from", "TPR"),
        ("one", "WUPB"),
        ("had", "H"),
        ("word", "WORBD"),
        ("but", "PWUT"),
        ("not", "TPHOT"),
        ("what", "WH"),
        ("all", "AUL"),
        ("were", "WR"),
        ("when", "WHEN"),
        ("can", "K"),
        ("said", "SED"),
        ("there", "THR"),
        ("each", "AOEFP"),
        ("she", "SHE"),
        ("how", "HOU"),
        ("will", "HR"),
        ("other", "OER"),
        ("about", "PW"),
        ("out", "OUT"),
        ("many", "PHAPB"),
        ("then", "THEPB"),
        ("them", "THEPL"),
        ("these", "THAEZ"),
        ("some", "SOPL"),
        ("would", "WO"),
        ("him", "HEUPL"),
        ("into", "TPHAO"),
        ("has", "HAS"),
        ("look", "HRAOBG"),
        ("two", "TWO"),
        ("more", "PHOR"),
        ("write", "WREU"),
        ("go", "GO"),
        ("see", "SAOE"),
        ("number", "TPHUR"),
        ("no", "TPHO"),
        ("way", "WAEU"),
        ("could", "KO"),
        ("people", "PAOEP"),
        ("my", "PHEU"),
        ("than", "THAPB"),
        ("first", "TPEURS"),
        ("water", "WAUR"),
        ("been", "PWEPB"),
        ("call", "KAUL"),
        ("who", "WHO"),
        ("oil", "OEUL"),
        ("its", "EUTS"),
        ("now", "TPHOU"),
        ("long", "HROPBG"),
        ("down", "TKOUPB"),
        ("day", "TKAEU"),
        ("did", "TK"),
        ("get", "TKPWET"),
        ("come", "KOPL"),
        ("made", "PHAED"),
        ("may", "PHAEU"),
        ("part", "PART"),
    ];

    DICT.iter()
        .map(|(w, o)| StenoPrompt {
            word: (*w).to_string(),
            outline: (*o).to_string(),
        })
        .collect()
}

pub fn load_custom_prompts() -> Option<Vec<StenoPrompt>> {
    let path = crate::config::paths::config_dir().join("steno.toml");
    let content = std::fs::read_to_string(path).ok()?;

    #[derive(serde::Deserialize)]
    struct Entry {
        word: String,
        outline: String,
    }

    #[derive(serde::Deserialize)]
    struct File {
        entries: Vec<Entry>,
    }

    let file: File = toml::from_str(&content).ok()?;
    if file.entries.is_empty() {
        return None;
    }

    if file.entries.iter().any(|entry| {
        entry.word.trim().is_empty() || entry.outline.trim().is_empty()
    }) {
        return None;
    }

    Some(
        file.entries
            .into_iter()
            .map(|entry| StenoPrompt {
                word: entry.word.trim().to_string(),
                outline: entry.outline.trim().to_string(),
            })
            .collect()
    )
}
