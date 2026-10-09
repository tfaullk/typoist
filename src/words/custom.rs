/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░█████╔╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░░╚═══██╗░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗██████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚═════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk


*/


use std::fs;
use std::path::PathBuf;

use crate::config::paths;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct CustomWordList {
    pub name: String,
    pub words: Vec<String>,
}

fn wordlists_dir() -> PathBuf {
    paths::wordlists_dir()
}

const README: &str = "\
Custom wordlists for typoist.

Drop .txt files in this directory. Each file becomes a word set you can
select with ctrl+w (or by setting word_set = \"custom:<filename>\" in
settings.toml).

Format:
  - One word or phrase per line
  - Lines starting with # are comments
  - Blank lines are ignored
  - A line can have a trailing # comment: `foo # note`
  - Case and symbols are preserved, so `Box<T>` and `kebab-case` work

Example file: spanish.txt
  # top spanish words
  el
  la
  de
  que
  y
";

pub fn ensure_dir() -> Result<()> {
    let dir = wordlists_dir();
    fs::create_dir_all(&dir)?;
    let readme = dir.join("README.md");
    if !readme.exists() {
        fs::write(readme, README)?;
    }
    Ok(())
}

pub fn parse(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| {
            match line.find('#') {
                Some(idx) => &line[..idx],
                None => line,
            }
        })
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn all() -> Result<Vec<CustomWordList>> {
    let dir = wordlists_dir();
    ensure_dir()?;
    let entries = fs::read_dir(&dir)?;
    let mut lists = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let is_txt = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| e.eq_ignore_ascii_case("txt"))
            .unwrap_or(false);
        if !is_txt {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        if stem.contains(':') {
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let words = parse(&content);
        if words.is_empty() {
            continue;
        }

        lists.push(CustomWordList {
            name: stem.to_string(),
            words,
        });
    }

    lists.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(lists)
}

pub fn names() -> Result<Vec<String>> {
    Ok(all()?.into_iter().map(|l| l.name).collect())
}

pub fn load(name: &str) -> Result<Option<CustomWordList>> {
    Ok(all()?.into_iter().find(|l| l.name == name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_list() {
        let words = parse("alpha\nbeta\ngamma\n");
        assert_eq!(words, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn ignores_comments_and_blanks() {
        let words = parse("# header\n\nalpha\n   \n# another\nbeta # trailing\n");
        assert_eq!(words, vec!["alpha", "beta"]);
    }

    #[test]
    fn preserves_case_and_symbols() {
        let words = parse("Box<T>\nfoo_bar\nkebab-case\n");
        assert_eq!(words, vec!["Box<T>", "foo_bar", "kebab-case"]);
    }
}
