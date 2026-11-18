/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

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
    pub update: bool,
    pub check_update: bool,
    pub export_pb: Option<String>,
    pub verify_pb: Option<PathBuf>,
    pub show_identity: bool,
    pub verify_pubkey: Option<String>,
}

impl CliArgs {
    pub fn parse() -> Self {
        let mut args = std::env::args_os()
            .skip(1)
            .map(|s| s.to_string_lossy().into_owned())
            .peekable();

        let mut out = Self {
            theme: None,
            word_set: None,
            mode: None,
            import_theme: None,
            list_themes: false,
            list_word_sets: false,
            no_save: false,
            update: false,
            check_update: false,
            export_pb: None,
            verify_pb: None,
            show_identity: false,
            verify_pubkey: None,
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
                "--update" => out.update = true,
                "--check-update" => out.check_update = true,
                "--version" | "-v" => {
                    println!("typoist {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--export-pb" => out.export_pb = args.next(),
                "--verify-pb" => out.verify_pb = args.next().map(PathBuf::from),
                "--identity" => out.show_identity = true,
                "--pubkey" => out.verify_pubkey = args.next(),
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

        if out.show_identity {
            let id = match crate::verify::Identity::load_or_create() { 
                Ok(id) => id,
                Err(e) => { eprintln!("typoist: {}", e); std::process::exit(1); } 
            };
            println!("public key: {}", id.public_hex());
            println!("key file:  {}", crate::config::paths::config_dir().join("identity.key").display());
            std::process::exit(0);
        }

        if let Some(mode) = out.export_pb {
            let chain = crate::verify::PbChain::open();
            match crate::verify::export::export_records(&chain, Some(&mode)) {
                Ok(jsons) => {
                    for j in jsons { println!("{}", j); }
                    std::process::exit(0);
                }
                Err(e) => { eprintln!("typoist: export failed: {}", e); std::process::exit(1); }
            }
        }

        if let Some(path) = out.verify_pb {
            let pubkey_hex = match out.verify_pubkey {
                Some(k) => k,
                None => {
                    eprintln!("typoist: --verify-pb requires --pubkey <hex>");
                    eprintln!("        get it with `typoist --identity` on the submission machine");
                    std::process::exit(1);
                }
            };
            let bytes = match hex::decode(&pubkey_hex) {
                Ok(b) if b.len() == 32 => b,
                _ => {
                    eprintln!("typoist: --pubkey must be 32 bytes of hex (64 chars)");
                    std::process::exit(1);
                }
            };
            let arr: [u8; 32] = bytes.try_into().unwrap();
            let verifying = match ed25519_dalek::VerifyingKey::from_bytes(&arr) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("typoist: --pubkey is not a valid Ed25519 public key");
                    std::process::exit(1);
                }
            };
            match crate::verify::export::verify_file(&path, &verifying) {
                Ok(report) => {
                    println!("{}", report.summary());
                    std::process::exit(if report.ok { 0 } else { 2 });
                }
                Err(e) => { eprintln!("typoist: {}", e); std::process::exit(1); }
            }
        }

        if out.update || out.check_update {
            match crate::update::run_update(out.check_update) {
                Ok(Some(outcome)) => {
                    // Distinct branch for "an update actually happened".
                    // For now, we just confirm and exit; if you later want
                    // to write to a log file or set a non-zero exit code
                    // for "was updated", this is the place.
                    eprintln!(
                        "typoist: updated {} -> {}",
                        outcome.from, outcome.to
                    );
                    std::process::exit(0);
                }
                Ok(None) => {
                    // No update needed, message already printed by run_update.
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("typoist: update failed: {}", e);
                    std::process::exit(1);
                }
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
    -v, --version             Show the current version
    --update                  Update in place
    --check-update            Check for updates without installing
    --verify-pb               Verifies personal best from hash (see more in README.md)
    --identity                Shows cryptographic identity for verification
    --export-pb               Exports PB (e.g typoist --export-pb "steno 30" > steno_30.jsonl)
    --pubkey                  Companion flag for --verify-pb, allows you to supply the ED25519   
                              key to verify. 

KEYBINDS:
    Default bindings (change from the in-app Keybinds screen):

    tab           restart test with new words
    esc           reset current test / back when idle
    ctrl+t        open theme picker
    ctrl+w        cycle word set
    ctrl+n        cycle mode
    ctrl+l        toggle live WPM
    ctrl+a        toggle accuracy display
    ctrl+g        toggle live WPM graph
    ctrl+h        open help
    ctrl+c        quit
"#
    );
}
