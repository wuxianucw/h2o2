use std::{fs, io, path::Path};

use super::utils;
use crate::{config, Com};

#[cfg(all(windows, target_arch = "x86"))]
pub(crate) const BIN_INFO: &str = "";

#[cfg(all(windows, target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "amd64.exe";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "amd64";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub(crate) const BIN_INFO: &str = "arm64";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "macOS-amd64";

pub async fn determine_mirror() -> Option<String> {
    let mirrors = vec!["https://github.com/", "https://download.fastgit.org/"];

    utils::determine_mirror(
        Com::Sandbox,
        mirrors,
        Some("wuxianucw/h2o2/releases/download/dummy/test"),
    )
    .await
    .map(|s| s + "criyle/go-judge/releases/download/v1.2.4/")
}

pub fn do_install(path: impl AsRef<Path>) -> io::Result<String> {
    let target_path = config::get_com_path().join("sandbox");
    fs::create_dir_all(&target_path)?;
    let target_path = target_path.join(if cfg!(windows) {
        "sandbox.exe"
    } else {
        "sandbox"
    });
    fs::copy(&path, &target_path)?;
    Ok(target_path.to_string_lossy().to_string())
}
