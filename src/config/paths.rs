/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░█████╔╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░░╚═══██╗░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗██████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚═════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use std::path::PathBuf;

// everything lives under <config_dir>/typoist, eg ~/.config/typoist on linux
pub fn config_dir() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        return dir.join("typoist");
    }

    #[cfg(windows)]
    { PathBuf::from("typoist") }

    #[cfg(not(windows))]
    { PathBuf::from(".config/typoist") }
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

pub fn ensure_config_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir())
}
