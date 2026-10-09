/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░█████╔╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░░╚═══██╗░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗██████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚═════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use super::custom;
use super::wordlists::*;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordSet {
    English,
    Code,
    EnglishPunctuation,
    CodeSymbols,
    /// Name of a file in `<config_dir>/typoist/wordlists/<name>.txt`
    Custom(String),
}

impl WordSet {
    /// Built-in variants, in the order they're shown/cycled first.
    pub fn builtins() -> &'static [WordSet] {
        &[
            WordSet::English,
            WordSet::Code,
            WordSet::EnglishPunctuation,
            WordSet::CodeSymbols,
        ]
    }

    /// Full ordered cycle used by `ctrl+w`: built-ins then customs.
    pub fn cycle() -> Vec<WordSet> {
        let mut out: Vec<WordSet> = Self::builtins().to_vec();
        if let Ok(names) = custom::names() {
            for name in names {
                out.push(WordSet::Custom(name));
            }
        }
        out
    }

    /// Stable string stored in settings.toml. Custom lists use `custom:<name>`
    /// so they can't collide with built-in labels.
    pub fn label(&self) -> String {
        match self {
            WordSet::English => "English".to_string(),
            WordSet::Code => "code".to_string(),
            WordSet::EnglishPunctuation => "EnglishPunctuation".to_string(),
            WordSet::CodeSymbols => "code+symbols".to_string(),
            WordSet::Custom(name) => format!("custom:{}", name),
        }
    }

    /// Pretty name for the header/footer.
    pub fn display_name(&self) -> String {
        match self {
            WordSet::Custom(name) => name.clone(),
            other => other.label(),
        }
    }

    /// Parse a settings/CLI string back into a `WordSet`.
    pub fn from_label(label: &str) -> Option<WordSet> {
        match label {
            "English" | "english" => Some(WordSet::English),
            "code" => Some(WordSet::Code),
            "EnglishPunctuation" | "english+punctuation" | "punct" => {
                Some(WordSet::EnglishPunctuation)
            }
            "code+symbols" | "symbols" => Some(WordSet::CodeSymbols),
            other => other
                .strip_prefix("custom:")
                .map(|name| WordSet::Custom(name.to_string())),
        }
    }

    /// Next set in the cycle, wrapping at the end.
    pub fn next(&self) -> WordSet {
        let cycle = Self::cycle();
        if cycle.is_empty() {
            return WordSet::English;
        }
        let idx = cycle.iter().position(|w| w == self).unwrap_or(0);
        cycle[(idx + 1) % cycle.len()].clone()
    }

    /// Only the punctuation-flavored built-ins get the cap/punct treatment.
    /// Custom lists are literal — what you wrote is what you type.
    pub fn adds_punctuation(&self) -> bool {
        matches!(
            self,
            WordSet::EnglishPunctuation | WordSet::CodeSymbols
        )
    }
}

pub struct WordGenerator {
    pub set: WordSet,
    /// Owned pool. For built-ins it's a copy of the static slice; for
    /// customs it's the loaded file contents.
    pool: Vec<String>,
    pub rng: rand::rngs::ThreadRng,
}

impl WordGenerator {
    pub fn new(set: WordSet) -> Self {
        let (set, pool) = match &set {
            WordSet::Custom(name) => match custom::load(name) {
                Ok(Some(list)) if !list.words.is_empty() => (set.clone(), list.words),
                _ => (
                    WordSet::English,
                    ENGLISH_200.iter().map(|s| s.to_string()).collect(),
                ),
            },
            WordSet::English | WordSet::EnglishPunctuation => (
                set.clone(),
                ENGLISH_200.iter().map(|s| s.to_string()).collect(),
            ),
            WordSet::Code | WordSet::CodeSymbols => (
                set.clone(),
                CODE_200.iter().map(|s| s.to_string()).collect(),
            ),
        };

        Self {
            set,
            pool,
            rng: rand::thread_rng(),
        }
    }

    pub fn generate(&mut self, count: usize) -> Vec<String> {
        if self.pool.is_empty() {
            return Vec::new();
        }

        let mut words: Vec<String> = (0..count)
            .map(|_| self.pool.choose(&mut self.rng).unwrap().clone())
            .collect();

        if self.set.adds_punctuation() {
            for (i, word) in words.iter_mut().enumerate() {
                if i == 0 || self.rng.gen_bool(0.15) {
                    let mut c = word.chars();
                    if let Some(first) = c.next() {
                        *word =
                            first.to_uppercase().collect::<String>() + c.as_str();
                    }
                }
                if self.rng.gen_bool(0.25) {
                    let p = PUNCTUATION_CHARS.choose(&mut self.rng).unwrap();
                    word.push(*p);
                }
            }
        }
        words
    }
}

