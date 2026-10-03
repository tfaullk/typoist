/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗     ██╗    ██████╗     ██████╗
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝    ███║   ██╔═████╗   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║       ╚██║   ██║██╔██║   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║        ██║   ████╔╝██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║        ██║██╗╚██████╔╝██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝        ╚═╝╚═╝ ╚═════╝ ╚═╝ ╚═════╝

Made with ♥ by tfaullk


*/

use super::wordlists::*;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSet {
    English,
    Code,
    EnglishPunctuation,
    CodeSymbols,
}

impl WordSet {
    pub fn variants() -> &'static [WordSet] {
        &[
            WordSet::English,
            WordSet::Code,
            WordSet::EnglishPunctuation,
            WordSet::CodeSymbols,
        ]
    }

    // these strings double as the names in settings.toml, keep them stable
    pub fn label(&self) -> &'static str {
        match self {
            WordSet::English => "English",
            WordSet::Code => "code",
            WordSet::EnglishPunctuation => "EnglishPunctuation",
            WordSet::CodeSymbols => "code+symbols",
        }
    }

    pub fn next(&self) -> WordSet {
        match self {
            WordSet::English => WordSet::Code,
            WordSet::Code => WordSet::EnglishPunctuation,
            WordSet::EnglishPunctuation => WordSet::CodeSymbols,
            WordSet::CodeSymbols => WordSet::English,
        }
    }
}

pub struct WordGenerator {
    pub set: WordSet,
    pub rng: rand::rngs::ThreadRng,
}

impl WordGenerator {
    pub fn new(set: WordSet) -> Self {
        Self {
            set,
            rng: rand::thread_rng(),
        }
    }

    // english variants share the english list, code variants share the code one
    fn base_words(&self) -> &'static [&'static str] {
        match self.set {
            WordSet::English | WordSet::EnglishPunctuation => ENGLISH_200,
            WordSet::Code | WordSet::CodeSymbols => CODE_200,
        }
    }

    pub fn generate(&mut self, count: usize) -> Vec<String> {
        let base = self.base_words();
        let mut words: Vec<String> = { 0..count }
            .map(|_| base.choose(&mut self.rng).unwrap().to_string())
            .collect();

        // punct sets get some spice: capitals and punctuation sprinkled in
        let add_punct = matches!(self.set, WordSet::EnglishPunctuation | WordSet::CodeSymbols);

        if add_punct {
            for (i, word) in words.iter_mut().enumerate() {
                // always capitalize the first word, looks like a real sentence
                if i == 0 || self.rng.gen_bool(0.15) {
                    let mut c = word.chars();
                    if let Some(first) = c.next() {
                        *word = first.to_uppercase().collect::<String>() + c.as_str();
                    }
                }

                // roughly a quarter of words get punctuation stuck on the end
                if self.rng.gen_bool(0.25) {
                    let p = PUNCTUATION_CHARS.choose(&mut self.rng).unwrap();
                    word.push(*p);
                }
            }
        }
        words
    }
}
