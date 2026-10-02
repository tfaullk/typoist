/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗     ██╗    ██████╗     ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝    ███║   ██╔═████╗   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║       ╚██║   ██║██╔██║   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║        ██║   ████╔╝██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║        ██║██╗╚██████╔╝██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝        ╚═╝╚═╝ ╚═════╝ ╚═╝ ╚═════╝

Made with ♥ by tfaullk


*/

pub mod theme;

pub use theme::Theme;

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::paths;
use crate::error::{Result, TypoistError};

// baked into the binary so a fresh install still has themes to show
const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("dracula.toml", include_str!("defaults/dracula.toml")),
    ("nord.toml", include_str!("defaults/nord.toml")),
    ("solarized_dark.toml", include_str!("defaults/solarized_dark.toml")),
    ("monokai.toml", include_str!("defaults/monokai.toml")),
    ("gruvbox.toml", include_str!("defaults/gruvbox.toml")),
    ("tokyonight.toml", include_str!("defaults/tokyonight.toml")),
    ("catppuccin-mocha.toml", include_str!("defaults/catppuccin-mocha.toml")),
    ("catppuccin-latte.toml", include_str!("defaults/catppuccin-latte.toml")),
    ("onedark.toml", include_str!("defaults/onedark.toml")),
    ("kanagawa.toml", include_str!("defaults/kanagawa.toml")),
    ("rose-pine.toml", include_str!("defaults/rose-pine.toml")),
    ("ayu-dark.toml", include_str!("defaults/ayu-dark.toml")),
    ("everforest.toml", include_str!("defaults/everforest.toml")),
    ("nightfox.toml", include_str!("defaults/nightfox.toml")),
    ("iceberg-dark.toml", include_str!("defaults/iceberg-dark.toml")),
    ("material-ocean.toml", include_str!("defaults/material-ocean.toml")),
    ("palenight.toml", include_str!("defaults/palenight.toml")),
    ("oceanic-next.toml", include_str!("defaults/oceanic-next.toml")),
    ("horizon.toml", include_str!("defaults/horizon.toml")),
    ("cga.toml", include_str!("defaults/cga.toml")),
];

fn themes_dir() -> PathBuf {
    paths::themes_dir()
}

// parse and give a decent error instead of a raw toml panic
fn parse_theme(content: &str, path: &Path) -> Result<Theme> {
    if content.trim().is_empty() {
        return Err(TypoistError::ThemeEmpty {
            path: path.display().to_string(),
        });
    }

    let theme: Theme = match toml::from_str(content) {
        Ok(theme) => theme,
        Err(error) => {
            let message = error.to_string();

            // missing field is a different problem than broken syntax, worth saying which
            if message.contains("missing field") {
                return Err(TypoistError::ThemeInvalid {
                    path: path.display().to_string(),
                    error: message,
                });
            }

            return Err(TypoistError::ThemeMalformed {
                path: path.display().to_string(),
                error: message,
            });
        }
    };

    theme.validate()?;

    Ok(theme)
}

// makes sure every built-in exists on disk and is valid,
// rewrites any that are missing or broken
fn ensure_theme_files() -> Result<()> {
    let dir = themes_dir();
    fs::create_dir_all(&dir)?;

    for (filename, content) in BUILTIN_THEMES {
        let path = dir.join(filename);

        // only clobber the user's file if it's actually broken
        let needs_restore = match fs::read_to_string(&path) {
            Ok(existing) => parse_theme(&existing, &path).is_err(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(error) => return Err(error.into()),
        };

        if needs_restore {
            let builtin_path = Path::new(filename);

            // if the built-in itself is broken that's on us, fail loudly
            let theme = parse_theme(content, builtin_path).map_err(|error| {
                TypoistError::BuiltinThemeInvalid(
                    filename.to_string(),
                    error.to_string(),
                )
            })?;

            let serialized = toml::to_string_pretty(&theme)?;

            fs::write(&path, serialized)?;
        }
    }

    Ok(())
}

// load everything in the themes dir, collecting errors instead of dying
// on the first bad file
fn load_user_themes() -> Result<(Vec<Theme>, Vec<String>)> {
    let dir = themes_dir();
    let entries = fs::read_dir(&dir)?;

    let mut themes = Vec::new();
    let mut errors = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("typoist: failed to read theme directory entry: {}", error);
                continue;
            }
        };

        let path = entry.path();

        // only .toml files are themes, ignore the rest
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                errors.push(format!(
                    "Could not read theme '{}': {}",
                    path.display(),
                    error
                ));
                continue;
            }
        };

        match parse_theme(&content, &path) {
            Ok(theme) => themes.push(theme),
            Err(error) => {
                errors.push(error.to_string());
            }
        }
    }

    Ok((themes, errors))
}

pub fn all() -> Result<(Vec<Theme>, Vec<String>)> {
    ensure_theme_files()?;

    let (mut themes, errors) = load_user_themes()?;

    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes.dedup_by(|a, b| a.name == b.name);

    // having literally zero themes means we can't even render an error screen nicely
    if themes.is_empty() {
        return Err(TypoistError::NoThemesAvailable);
    }

    Ok((themes, errors))
}

pub fn list_names() -> Result<Vec<String>> {
    Ok(all()?.0.into_iter().map(|theme| theme.name).collect())
}

pub fn import_theme(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    // validate before copying, no point importing a broken theme
    let theme = parse_theme(&content, path)?;

    ensure_theme_files()?;

    let destination = themes_dir().join(format!("{}.toml", theme.name));
    fs::write(destination, content)?;

    Ok(())
}
