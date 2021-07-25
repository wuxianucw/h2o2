use super::utils;
use crate::Com;

#[cfg(all(windows, target_arch = "x86"))]
pub(crate) const BIN_INFO: &str = "";

#[cfg(all(windows, target_arch = "x86_64"))]
pub(crate) const BIN_INFO: &str = "windows-amd64/minio.exe";

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub(crate) const BIN_INFO: &str = "linux-amd64/minio";

#[cfg(all(target_os = "linux", target_arch = "arm"))]
pub(crate) const BIN_INFO: &str = "linux-arm64/minio";

#[cfg(target_os = "macos")]
pub(crate) const BIN_INFO: &str = "darwin-amd64/minio";

pub async fn determine_mirror() -> Option<String> {
    let mirrors = vec![
        "http://dl.min.io/server/minio/release/",
        "http://dl.minio.org.cn/server/minio/release/",
    ];

    utils::determine_mirror(Com::MinIO, mirrors, None).await
}
