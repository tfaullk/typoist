/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░░███╔═╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░██╔══╝░░░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗███████╗██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚══════╝╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use crate::config::Settings;
use crate::engine::TestMode;
use crate::themes;
use crate::words::WordSet;
use std::path::PathBuf;

pub struct CliArgs {
    pub theme: Option<String>,
    pub word_set: Option<WordSet>,
    pub mode: Option<TestMode>,
    pub list_themes: bool,
    pub list_word_sets: bool,
    pub no_save: bool,
    pub import_theme: Option<PathBuf>,
}

impl CliArgs {
    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1).peekable();
        let mut out = Self {
            theme: None,
            word_set: None,
            mode: None,
            import_theme: None,
            list_themes: false,
            list_word_sets: false,
            no_save: false,
        };

        while let Some(a) = args.next() {
            match a.as_str() {
                "--theme" | "-t" => out.theme = args.next(),
                "--words" | "-w" => {
                    out.word_set = args
                        .next()
                        .and_then(|s| WordSet::from_label(&s));
                }
                "--mode" | "-m" => {
                    // accept "time 30", "words 25", "time30", or just "30"
                    out.mode = match args.next().as_deref() {
                        Some("quote") => Some(TestMode::Quote),
                        Some(s) if s.starts_with("time") => {
                            let n = s.trim_start_matches("time").trim().parse().ok();
                            n.map(TestMode::Time)
                        }
                        Some(s) if s.starts_with("words") => {
                            let n = s.trim_start_matches("words").trim().parse().ok();
                            n.map(TestMode::Words)
                        }
                        Some(s) => s.parse().ok().map(TestMode::Time),
                        None => None,
                    }
                }
                "--list-themes" => out.list_themes = true,
                "--list-word-sets" => out.list_word_sets = true,
                "--no-save" => out.no_save = true,
                "--import-theme" => {
                    out.import_theme = args.next().map(PathBuf::from);
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        // import runs before the tui opens, it's a one-shot thing
        if let Some(path) = &out.import_theme {
            if let Err(e) = themes::import_theme(path) {
                eprintln!("typoist: failed to import theme: {}", e);
                std::process::exit(1);
            }
        }

        // these are all "print stuff and leave" flags
        if out.list_themes {
            match themes::list_names() {
                Ok(names) => {
                    for name in names {
                        println!("{}", name);
                    }
                }
                Err(e) => {
                    eprintln!("typoist: failed to load themes: {}", e);
                    std::process::exit(1);
                }
            }

            std::process::exit(0);
        }

        if out.list_word_sets {
            println!("Built-in:");
            for w in WordSet::builtins() {
                println!("  {}", w.label());
            }
            match crate::words::custom::names() {
                Ok(names) if !names.is_empty() => {
                    println!("\nCustom ({}):", crate::config::paths::wordlists_dir().display());
                    for name in names {
                        println!("  custom:{}", name);
                    }
                }
                Ok(_) => {
                    println!(
                        "\nNo custom wordlists. Drop .txt files (one word per line) into:\n  {}",
                        crate::config::paths::wordlists_dir().display()
                    );
                }
                Err(e) => eprintln!("(failed to scan wordlists: {})", e),
            }
            std::process::exit(0);
        }
        out
    }

    // only the flags that were actually passed get written into settings
    pub fn apply_to(self, mut s: Settings) -> Settings {
        if let Some(t) = self.theme {
            s.theme = t;
        }
        if let Some(w) = self.word_set {
            s.word_set = w.label().to_string();
        }
        if let Some(m) = self.mode {
            s.mode = m.label();
        }
        s
    }
}

fn print_help() {
    println!(
        r#"typoist — a CLI typing test

USAGE:
    typoist [OPTIONS]

OPTIONS:
    -t, --theme <NAME>        Start with a specific theme
    -w, --words <SET>         Word set: english | code | english+punctuation | code+symbols
    -m, --mode <MODE>         Mode: time 15|30|60|120 | words 10|25|50|100 | quote
        --list-themes         List available themes and exit
        --list-word-sets      List available word sets and exit
        --no-save             Don't persist settings this session
    -h, --help                Show this help

IN-TEST KEYS:
    tab       new test
    esc       reset / back
    t         theme picker
    w         cycle word set
    m         cycle mode
    l         toggle live wpm
    a         toggle accuracy
    ?         help
    ctrl+c    quit
"#
    );
}
