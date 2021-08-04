use duct::cmd;
use std::{env, io, path::Path};

use super::utils;
use crate::Com;

#[cfg(all(windows, target_arch = "x86"))]
pub(crate) const BIN_INFO: (&str, &str) = (
    "-x86.msi",
    "b5bea503f45058a6acd0900bfe7e52deba12dcc1769808eece93b42bce40c7d8",
);

#[cfg(all(windows, target_arch = "x86_64"))]
pub(crate) const BIN_INFO: (&str, &str) = (
    "-x64.msi",
    "964e36aa518b17ab04c3a49a0f5641a6bd8a9dc2b57c18272b6f90edf026f5dc",
);

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) const BIN_INFO: (&str, &str) = (
    "-linux-x64.tar.gz",
    "7ef1f7dae52a3ec99cda9cf29e655bc6e61c2c48e496532d83d9f17ea108d5d8",
);

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub(crate) const BIN_INFO: (&str, &str) = (
    "-linux-arm64.tar.gz",
    "784ede0c9faa4a71d77659918052cca39981138edde2c799ffdf2b4695c08544",
);

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub(crate) const BIN_INFO: (&str, &str) = (
    "-darwin-x64.tar.gz",
    "522f85db1d1fe798cba5f601d1bba7b5203ca8797b2bc934ff6f24263f0b7fb2",
);

pub async fn determine_mirror() -> Option<String> {
    let mirrors = vec![
        "https://nodejs.org/dist/",
        "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/",
        "https://mirrors.cloud.tencent.com/nodejs-release/",
    ];
    let testfile = "v14.17.3/SHASUMS256.txt";

    utils::determine_mirror(Com::NodeJS, mirrors, Some(testfile)).await
}

#[cfg(windows)]
pub fn do_install(path: impl AsRef<Path>) -> io::Result<String> {
    // msiexec /i <file> /quiet /qn /norestart
    if !cfg!(debug_assertions) {
        cmd!(
            "msiexec",
            "/i",
            path.as_ref(),
            "/quiet",
            "/qn",
            "/norestart"
        )
        .stdout_capture()
        .stderr_capture()
        .run()?;
    }

    let path = env::var("PROGRAMFILES").unwrap();
    let path = Path::new(&path).join("nodejs").join("node.exe");
    Ok(path.to_string_lossy().to_string())
}

#[cfg(unix)]
pub fn do_install(path: impl AsRef<Path>) -> io::Result<String> {
    use crate::config;
    use std::fs::create_dir_all;
    use std::os::unix::fs::symlink;

    // tar -xzf <file> -C <path>
    let target_path = config::get_com_path().join("nodejs");
    let _ = create_dir_all(&target_path);
    cmd!("tar", "-xzf", path.as_ref(), "-C", target_path.as_ref())
        .stdout_capture()
        .stderr_capture()
        .run()?;

    if !cfg!(debug_assertions) {
        // symlink to /usr/local/bin/node
        let path = target_path.join("node");
        symlink(&path, "/usr/local/bin/node")?;
    }

    Ok(path.to_string_lossy().to_string())
}
