use std::{fs, io, path::Path};

use super::utils;
use crate::{config, Com};

#[cfg(all(windows, target_arch = "x86"))]
pub(crate) const BIN_INFO: &str = "";

#[cfg(all(windows, target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "windows-amd64/minio.exe";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "linux-amd64/minio";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub(crate) const BIN_INFO: &str = "linux-arm64/minio";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "darwin-amd64/minio";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) const BIN_INFO: &str = "darwin-arm64/minio";

pub async fn determine_mirror() -> Option<String> {
    let mirrors = vec![
        "http://dl.min.io/server/minio/release/",
        "http://dl.minio.org.cn/server/minio/release/",
    ];

    utils::determine_mirror(Com::MinIO, mirrors, None).await
}

pub fn do_install(path: impl AsRef<Path>) -> io::Result<String> {
    let target_path = config::get_com_path().join("minio");
    fs::create_dir_all(&target_path)?;
    let target_path = target_path.join(if cfg!(windows) { "minio.exe" } else { "minio" });
    fs::copy(&path, &target_path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(&target_path)?.permissions();
        let mode = perms.mode() | 0o111;
        perms.set_mode(mode);
        fs::set_permissions(&target_path, perms)?;
    }
    Ok(target_path.to_string_lossy().into_owned())
}
