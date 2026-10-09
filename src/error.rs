/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░█████╔╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░░╚═══██╗░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗██████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚═════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk


*/


use thiserror::Error;

// one error type for everything so main() can just `?` its way through
#[derive(Error, Debug)]
pub enum TypoistError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Theme file '{path}' is empty")]
    ThemeEmpty { path: String },

    #[error("Theme file '{path}' is incomplete or contains invalid fields: {error}")]
    ThemeInvalid { path: String, error: String },

    #[error("Theme file '{path}' contains malformed TOML: {error}")]
    ThemeMalformed { path: String, error: String },

    #[error("Theme '{0}' has an empty name")]
    ThemeEmptyName(String),

    #[error("Theme '{name}' has an invalid name: {reason}")]
    ThemeInvalidName { name: String, reason: String },

    // a built-in failing is always a bug, not a user mistake
    #[error("Built-in theme '{0}' is invalid: {1}")]
    BuiltinThemeInvalid(String, String),

    #[error("No valid themes are available")]
    NoThemesAvailable,

    #[error("Serialization Error: {0}")]
    Serialize(#[from] toml::de::Error),

    #[error("Failed to write config: {0}")]
    SerializeWrite(#[from] toml::ser::Error),

    #[error("Update check failed: {0}")]
    UpdateCheck(String),

    #[error("No release found for your platform ({target})")]
    UpdateNoArtifact { target: String },

    #[error("Update download failed: {0}")]
    UpdateDownload(String),

    #[error("Update install failed: {0}")]
    UpdateInstall(String),

    #[error("Could not determine the current executable path: {0}")]
    UpdateCurrentExe(String),
}

pub type Result<T> = std::result::Result<T, TypoistError>;
