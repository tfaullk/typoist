/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░█████╔╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░░╚═══██╗░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗██████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚═════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

pub mod theme;

pub use theme::Theme;

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::paths;
use crate::error::{Result, TypoistError};

#[derive(Debug, Clone)]
pub struct ThemeWarning {
    pub title: String,
    pub message: String,
    pub path: Option<String>,
    pub details: Option<String>,
}

impl ThemeWarning {
    fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        path: Option<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            path,
            details,
        }
    }
}

// baked into the binary so a fresh install still has themes to show
const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("dracula.toml", include_str!("defaults/dracula.toml")),
    ("nord.toml", include_str!("defaults/nord.toml")),
    (
        "solarized_dark.toml",
        include_str!("defaults/solarized_dark.toml"),
    ),
    ("monokai.toml", include_str!("defaults/monokai.toml")),
    ("gruvbox.toml", include_str!("defaults/gruvbox.toml")),
    ("tokyonight.toml", include_str!("defaults/tokyonight.toml")),
    (
        "catppuccin-mocha.toml",
        include_str!("defaults/catppuccin-mocha.toml"),
    ),
    (
        "catppuccin-latte.toml",
        include_str!("defaults/catppuccin-latte.toml"),
    ),
    ("onedark.toml", include_str!("defaults/onedark.toml")),
    ("kanagawa.toml", include_str!("defaults/kanagawa.toml")),
    ("rose-pine.toml", include_str!("defaults/rose-pine.toml")),
    ("ayu-dark.toml", include_str!("defaults/ayu-dark.toml")),
    ("everforest.toml", include_str!("defaults/everforest.toml")),
    ("nightfox.toml", include_str!("defaults/nightfox.toml")),
    (
        "iceberg-dark.toml",
        include_str!("defaults/iceberg-dark.toml"),
    ),
    (
        "material-ocean.toml",
        include_str!("defaults/material-ocean.toml"),
    ),
    ("palenight.toml", include_str!("defaults/palenight.toml")),
    (
        "oceanic-next.toml",
        include_str!("defaults/oceanic-next.toml"),
    ),
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

fn theme_warning_from_error(error: &TypoistError) -> ThemeWarning {
    match error {
        TypoistError::ThemeEmpty { path } => ThemeWarning::new(
            "Theme file is empty",
            "This theme cannot be loaded because it does not contain any settings.\n\
             Copy an existing theme file and edit its colours, or remove this file.",
            Some(path.clone()),
            None,
        ),

        TypoistError::ThemeInvalid { path, error } => {
            let message = match missing_theme_field(error) {
                Some(field) => format!(
                    "This theme is missing the required '{}' setting.\n\
                     Add that setting by copying it from an existing theme file.",
                    field
                ),
                None => String::from(
                    "This theme has an invalid or incomplete setting.\n\
                     Each colour must use three RGB values, for example:\n\
                     accent = [139, 233, 253]",
                ),
            };

            ThemeWarning::new(
                "Theme settings are incomplete",
                message,
                Some(path.clone()),
                Some(error.clone()),
            )
        }

        TypoistError::ThemeMalformed { path, error } => ThemeWarning::new(
            "Theme file has invalid TOML",
            "Check for a missing quote, comma, bracket, or '=' sign.\n\
             You can copy an existing theme as a working template.",
            Some(path.clone()),
            Some(error.clone()),
        ),

        TypoistError::ThemeEmptyName(name) => ThemeWarning::new(
            "Theme name is missing",
            "Add a name at the top of the file, for example:\n\
             name = \"my-theme\"",
            None,
            Some(format!("Received theme name: {:?}", name)),
        ),

        TypoistError::ThemeInvalidName { name, reason } => ThemeWarning::new(
            "Theme name cannot be used",
            format!(
                "The theme name '{}' is not valid.\n\
                 Use a simple file-safe name such as 'my-theme'.",
                name
            ),
            None,
            Some(reason.clone()),
        ),

        TypoistError::Io(error) => ThemeWarning::new(
            "Theme file could not be read",
            "typoist could not access a theme file.\n\
             Check that the theme directory and file are readable.",
            None,
            Some(error.to_string()),
        ),

        TypoistError::BuiltinThemeInvalid(filename, error) => ThemeWarning::new(
            "Built-in theme is invalid",
            format!(
                "The built-in theme '{}' could not be loaded.\n\
                 This is likely an installation problem. Reinstall typoist.",
                filename
            ),
            None,
            Some(error.clone()),
        ),

        TypoistError::NoThemesAvailable => ThemeWarning::new(
            "No usable themes found",
            "typoist could not find a valid theme to display.\n\
             Restore the themes directory or reinstall typoist.",
            None,
            None,
        ),

        TypoistError::Serialize(error) => ThemeWarning::new(
            "Theme could not be parsed",
            "typoist could not read theme data.",
            None,
            Some(error.to_string()),
        ),

        TypoistError::SerializeWrite(error) => ThemeWarning::new(
            "Theme configuration could not be saved",
            "Check that the typoist configuration directory is writable.",
            None,
            Some(error.to_string()),
        ),

        other => ThemeWarning::new(
            "Unexpected error",
            "typoist hit an internal error while loading themes.\n\
             This is a bug; please report it",
            None,
            Some(format!("{:?}", other)),
        ),
    }
}

fn missing_theme_field(error: &str) -> Option<&str> {
    let marker = "missing field `";
    let start = error.find(marker)? + marker.len();
    let remaining = &error[start..];
    let end = remaining.find('`')?;
    Some(&remaining[..end])
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
            Ok(_) => false,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(error) => return Err(error.into()),
        };

        if needs_restore {
            let builtin_path = Path::new(filename);

            // if the built-in itself is broken that's on us, fail loudly
            let theme = parse_theme(content, builtin_path).map_err(|error| {
                TypoistError::BuiltinThemeInvalid(filename.to_string(), error.to_string())
            })?;

            let serialized = toml::to_string_pretty(&theme)?;

            fs::write(&path, serialized)?;
        }
    }

    Ok(())
}

// load everything in the themes dir, collecting errors instead of dying
// on the first bad file
fn load_user_themes() -> Result<(Vec<Theme>, Vec<ThemeWarning>)> {
    let dir = themes_dir();
    let entries = fs::read_dir(&dir)?;

    let mut themes = Vec::new();
    let mut warnings = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warnings.push(ThemeWarning::new(
                    "A theme directory could not be read",
                    "typoist skipped one item in the themes directory.",
                    None,
                    Some(error.to_string()),
                ));
                continue;
            }
        };

        let path = entry.path();

        let is_toml = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| e.eq_ignore_ascii_case("toml"))
            .unwrap_or(false);
        if !is_toml {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                warnings.push(ThemeWarning::new(
                    "Theme file could not be read",
                    "typoist skipped this file. Check that it still exists and that you have permission to read it.",
                    Some(path.display().to_string()),
                    Some(error.to_string()),
                ));
                continue;
            }
        };

        match parse_theme(&content, &path) {
            Ok(theme) => themes.push(theme),
            Err(error) => {
                warnings.push(theme_warning_from_error(&error));
            }
        }
    }
    Ok((themes, warnings))
}

pub fn all() -> Result<(Vec<Theme>, Vec<ThemeWarning>)> {
    ensure_theme_files()?;

    let (mut themes, warnings) = load_user_themes()?;

    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes.dedup_by(|a, b| a.name == b.name);

    // having literally zero themes means we can't even render an error screen nicely
    if themes.is_empty() {
        return Err(TypoistError::NoThemesAvailable);
    }

    Ok((themes, warnings))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_field_becomes_friendly_warning() {
        let error = TypoistError::ThemeInvalid {
            path: "/tmp/broken-theme.toml".into(),
            error: "TOML parse error: missing field `accent`".into(),
        };

        let warning = theme_warning_from_error(&error);

        assert_eq!(warning.title, "Theme settings are incomplete");
        assert!(warning.message.contains("accent"));
        assert_eq!(warning.path.as_deref(), Some("/tmp/broken-theme.toml"));
    }

    #[test]
    fn malformed_toml_has_recovery_advice() {
        let error = TypoistError::ThemeMalformed {
            path: "/tmp/broken-theme.toml".into(),
            error: "TOML parse error at line 4, column 1".into(),
        };

        let warning = theme_warning_from_error(&error);

        assert_eq!(warning.title, "Theme file has invalid TOML");
        assert!(warning.message.contains("missing quote"));
        assert_eq!(warning.path.as_deref(), Some("/tmp/broken-theme.toml"));
    }

    #[test]
    fn missing_field_helper_extracts_name() {
        assert_eq!(
            missing_theme_field("TOML parse error: missing field `cursor`"),
            Some("cursor")
        );
    }
}
