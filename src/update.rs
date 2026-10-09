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
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;

use crate::error::{Result, TypoistError};

const REPO: &str = "tfaullk/typoist";
const USER_AGENT: &str = concat!("typoist/", env!("CARGO_PKG_VERSION"));

fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION must be valid semver")
}

fn repo_slug() -> String {
    std::env::var("TYPOIST_UPDATE_REPO").unwrap_or_else(|_| REPO.to_string())
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

fn parse_tag(tag: &str) -> Option<Version> {
    let trimmed = tag.trim_start_matches('v');
    Version::parse(trimmed).ok()
}

pub fn current_target() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "x86_64-unknown-linux-musl";
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return "aarch64-unknown-linux-musl";
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "x86_64-pc-windows-msvc";
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "x86_64-apple-darwin";
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "aarch64-apple-darwin";
    }

    #[allow(unreachable_code)]
    {
        "unsupported"
    }
}

fn artifact_name(version: &Version, target: &str) -> String {
    let ext = if target.contains("windows") { ".exe" } else { "" };
    format!("typoist-{}-{}{}", version, target, ext)
}

fn fetch_latest() -> Result<GhRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        repo_slug()
    );

    let resp = ureq::get(&url) 
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| TypoistError::UpdateCheck(e.to_string()))?;

    resp.into_json::<GhRelease>()
        .map_err(|e| TypoistError::UpdateCheck(e.to_string()))
}

fn download_asset(asset: &GhAsset, dest: &Path) -> Result<()> {
    let resp = ureq::get(&asset.browser_download_url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| TypoistError::UpdateDownload(e.to_string()))?;

    let mut reader = resp.into_reader();
    let mut file = fs::File::create(dest)
        .map_err(|e| TypoistError::UpdateDownload(e.to_string()))?;

    let mut buf = [0u8; 8192];
    let mut total: u64 = 0;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| TypoistError::UpdateDownload(e.to_string()))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| TypoistError::UpdateDownload(e.to_string()))?;
        total += n as u64;
    }
    if total != asset.size {
        return Err(TypoistError::UpdateDownload(format!(
            "expected {} bytes, got {}",
            asset.size, total
        )));
    }

    Ok(())
}

fn current_exe() -> Result<PathBuf> {
    std::env::current_exe().map_err(|e| TypoistError::UpdateCurrentExe(e.to_string()))
}

fn install_binary(src: &Path, dst: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(src)
            .map_err(|e| TypoistError::UpdateInstall(e.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(src, perms)
            .map_err(|e| TypoistError::UpdateInstall(e.to_string()))?;

        fs::rename(src, dst).map_err(|e| TypoistError::UpdateInstall(e.to_string()))?;
    }

    #[cfg(windows)]
    {
        let old = dst.with_extension("exe.old");
        let _ = fs::remove_file(&old);

        fs::rename(dst, &old).map_err(|e| {
            TypoistError::UpdateInstall(format!(
                "could not move running exe aside (try running as admin perhaps): {}",
                e
            ))
        })?;

        if let Err(e) = fs::rename(src, dst) {
            let _ = fs::rename(&old, dst);
            return Err(TypoistError::UpdateInstall(e.to_string()));
        }
    }

    Ok(())
}

pub fn cleanup_previous_update() {
    #[cfg(windows)]
    {
        if let Ok(exe) = current_exe() {
            let old = exe.with_extension("exe.old");
            let _ = fs::remove_file(&old);
        }
    }
}

pub struct UpdateOutcome {
    pub from: Version,
    pub to: Version,
}

pub fn run_update(check_only: bool) -> Result<Option<UpdateOutcome>> {
    let current = current_version();

    println!("typoist: checking for updates...");
    println!("  current version: {}", current);

    let release = fetch_latest()?;
    let latest = parse_tag(&release.tag_name).ok_or_else(|| {
        TypoistError::UpdateCheck(format!("could not parse tag {:?}", release.tag_name))
    })?;

    println!("  latest version:  {}", latest);

    if latest <= current {
        println!();
        println!("typoist is already up to date.");
        return Ok(None);
    }

    if check_only {
        println!();
        println!("A new version is available: {} -> {}", current, latest);
        println!("Run `typoist --update` to install it.");
        return Ok(Some(UpdateOutcome {
            from: current,
            to: latest,
        }));
    }

    let target = current_target();
    let wanted = artifact_name(&latest, target);

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == wanted) 
        .ok_or_else(|| TypoistError::UpdateNoArtifact {
            target: target.to_string(),
        })?;

    println!();
    println!(
        "Downloading {} ({:1} MiB)...",
        asset.name,
        asset.size as f64 / 1024.0 / 1024.0
    );

    let exe = current_exe()?;
    let staging = exe.with_extension("new");
    download_asset(asset, &staging)?;

    println!("Installing...");
    install_binary(&staging, &exe)?;

    println!();
    println!("Updated typoist: {} -> {}", current, latest);

    #[cfg(windows)]
    println!("Please restart typoist for the change to take effect.");

    Ok(Some(UpdateOutcome {
        from: current,
        to: latest,
    }))
}

#[allow(dead_code)]
pub fn check_for_update_quietly() -> Option<Version>{
    let current = current_version();
    let release = fetch_latest().ok()?;
    let latest = parse_tag(&release.tag_name)?;
    if latest > current {
        Some(latest)
    } else {
        None
    }
}
