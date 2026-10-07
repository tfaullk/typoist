/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░░███╔═╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░██╔══╝░░░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗███████╗██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚══════╝╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use std::path::PathBuf;

// everything lives under <config_dir>/typoist, eg ~/.config/typoist on linux
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("typoist")
}

pub fn settings_file() -> PathBuf {
    config_dir().join("settings.toml")
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

pub fn wordlists_dir() -> PathBuf {
    config_dir().join("wordlists")
}
