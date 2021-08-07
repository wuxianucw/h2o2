use derive_more::{Constructor, Display, IsVariant};
use std::result::Result as StdResult;
use thiserror::Error as ThisError;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    sync::broadcast::{error::RecvError, Receiver},
    time,
};

use super::helper::*;
pub use crate::config::ComponentInfo;
use crate::{config::Version, utils::sha256_file};

#[derive(ThisError, Debug, Constructor)]
#[error("Failed to install {com}: {kind}")]
pub struct Error {
    pub com: Com,
    #[source]
    pub kind: ErrorKind,
}

#[derive(Debug, Display, ThisError)]
pub enum ErrorKind {
    // TODO: more error kind
    #[display(fmt = "{}", _0)]
    RecvError(#[from] RecvError),

    #[display(fmt = "require {}", _0)]
    DependencyError(Com),

    #[display(fmt = "your platform is not supported yet")]
    PlatformNotSupported,

    #[display(fmt = "no available source")]
    NoAvailableSource,

    #[display(fmt = "{}", _0)]
    IOError(#[from] std::io::Error),

    #[display(fmt = "{}", _0)]
    RequestError(#[from] reqwest::Error),

    #[display(fmt = "{}", _0)]
    RespError(reqwest::StatusCode),

    #[display(fmt = "file checksum mismatch")]
    ChecksumMismatch,

    #[display(fmt = "{}", _0)]
    Other(String),
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Eq)]
pub enum Com {
    #[display(fmt = "Node.js")]
    NodeJS,
    #[display(fmt = "MongoDB")]
    MongoDB,
    #[display(fmt = "MinIO")]
    MinIO,
    #[display(fmt = "sandbox")]
    Sandbox,
    #[display(fmt = "Yarn")]
    Yarn,
    #[display(fmt = "PM2")]
    PM2,
    #[display(fmt = "Hydro")]
    Hydro,
}

#[derive(Debug, IsVariant, Clone)]
pub enum Signal<'a> {
    Ready(Com, &'a ComponentInfo),
    Failed(Com),
}

macro_rules! wait_for_components {
    ($com:expr, $rx:expr, $($dep_com:expr),+ $(,)?) => {{
        let mut coms = vec![$($dep_com),+];
        while !coms.is_empty() {
            match $rx.recv().await.map_err(|e| Error::new($com, ErrorKind::RecvError(e)))? {
                Signal::Ready(com, _info) => {
                    // TODO: collect ComponentInfo
                    if let Some(pos) = coms.iter().position(|x| *x == com) {
                        coms.swap_remove(pos);
                    }
                }
                Signal::Failed(com) => {
                    if let Some(_) = coms.iter().position(|x| *x == com) {
                        return Err(Error::new($com, ErrorKind::DependencyError(com)));
                    }
                }
            }
        }
    }};
}

pub type Result<T> = StdResult<T, Error>;

pub async fn install(com: Com, rx: Option<Receiver<Signal<'_>>>) -> Result<(Com, ComponentInfo)> {
    match com {
        // must await each, because `impl Future<Output = T>` is an opaque type
        Com::NodeJS => install_nodejs().await,
        Com::MongoDB => install_mongodb().await,
        Com::MinIO => install_minio().await,
        Com::Sandbox => install_sandbox().await,
        Com::Yarn => {
            let mut rx = rx.expect("Receiver cannot be `None`");
            wait_for_components!(com, rx, Com::NodeJS);
            install_yarn().await
        }
        Com::PM2 => {
            let mut rx = rx.expect("Receiver cannot be `None`");
            wait_for_components!(com, rx, Com::NodeJS);
            install_pm2().await
        }
        Com::Hydro => {
            let mut rx = rx.expect("Receiver cannot be `None`");
            wait_for_components!(com, rx, Com::NodeJS, Com::Yarn);
            install_hydro().await
        }
    }
    .map(|ok| (com, ok))
    .map_err(|e| Error::new(com, e))
}

type InstallResult<T> = StdResult<T, ErrorKind>;

async fn install_nodejs() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 Node.js... Start to install Node.js...");

    log::info!("[Node.js] 寻找最快的下载源... Finding the fastest download source...");
    let dist = nodejs::determine_mirror()
        .await
        .ok_or(ErrorKind::NoAvailableSource)?;
    let (postfix, shasum256) = nodejs::BIN_INFO;
    let filename = format!("node-v14.17.3{}", postfix);
    let url = format!("{}v14.17.3/{}", &dist, &filename);
    log::info!("[Node.js] {}", &url);

    let dir = tempfile::tempdir().map_err(ErrorKind::IOError)?;
    let path = dir.path().join(&filename);
    let mut file = File::create(&path).await.map_err(ErrorKind::IOError)?;

    log::info!("[Node.js] 开始下载... Downloading...");
    let mut res = reqwest::get(url).await.map_err(ErrorKind::RequestError)?;
    if !res.status().is_success() {
        return Err(ErrorKind::RespError(res.status()));
    }

    while let Some(chunk) = res.chunk().await.map_err(ErrorKind::RequestError)? {
        file.write_all(&chunk).await.map_err(ErrorKind::IOError)?;
    }

    file.sync_all().await.map_err(ErrorKind::IOError)?;
    log::info!("[Node.js] 下载完毕。 Download completed.");

    if sha256_file(&path).map_err(ErrorKind::IOError)? != shasum256 {
        log::info!("[Node.js] 文件校验失败！ File checksum mismatch!");
        return Err(ErrorKind::ChecksumMismatch);
    }

    let path = nodejs::do_install(&path).map_err(ErrorKind::IOError)?;

    Ok(ComponentInfo::new(
        Version::Valid(semver::Version::parse("14.17.3").unwrap()),
        Some(path),
    ))
}

async fn install_mongodb() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 MongoDB... Start to install MongoDB...");

    if cfg!(target_arch = "x86") {
        log::error!("[MongoDB] x86 架构不受支持。 The x86 architecture is not supported.");
        return Err(ErrorKind::PlatformNotSupported);
    }

    time::sleep(time::Duration::from_secs(20)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}

async fn install_minio() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 MinIO... Start to install MinIO...");

    if cfg!(target_arch = "x86") {
        log::error!("[MinIO] x86 架构不受支持。 The x86 architecture is not supported.");
        return Err(ErrorKind::PlatformNotSupported);
    }

    log::info!("[MinIO] 寻找最快的下载源... Finding the fastest download source...");
    let dist = minio::determine_mirror()
        .await
        .ok_or(ErrorKind::NoAvailableSource)?;
    let file = minio::BIN_INFO;
    let url = format!("{}{}", &dist, file);

    log::info!("[MinIO] {}", &url);

    let dir = tempfile::tempdir().map_err(ErrorKind::IOError)?;
    let path = dir.path().join("minio");
    let mut file = File::create(&path).await.map_err(ErrorKind::IOError)?;

    log::info!("[MinIO] 开始下载... Downloading...");
    let mut res = reqwest::get(url).await.map_err(ErrorKind::RequestError)?;
    if !res.status().is_success() {
        return Err(ErrorKind::RespError(res.status()));
    }

    while let Some(chunk) = res.chunk().await.map_err(ErrorKind::RequestError)? {
        file.write_all(&chunk).await.map_err(ErrorKind::IOError)?;
    }

    file.sync_all().await.map_err(ErrorKind::IOError)?;
    log::info!("[MinIO] 下载完毕。 Download completed.");

    let path = minio::do_install(&path).map_err(ErrorKind::IOError)?;

    Ok(ComponentInfo::new(Version::Installed, Some(path)))
}

async fn install_sandbox() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 sandbox... Start to install sandbox...");

    if cfg!(target_arch = "x86") {
        log::error!("[sandbox] x86 架构不受支持。 The x86 architecture is not supported.");
        return Err(ErrorKind::PlatformNotSupported);
    }

    log::info!("[sandbox] 寻找最快的下载源... Finding the fastest download source...");
    let dist = sandbox::determine_mirror()
        .await
        .ok_or(ErrorKind::NoAvailableSource)?;
    let postfix = sandbox::BIN_INFO;
    let url = format!("{}executorserver-{}", &dist, postfix);

    log::info!("[sandbox] {}", &url);

    let dir = tempfile::tempdir().map_err(ErrorKind::IOError)?;
    let path = dir.path().join("sandbox");
    let mut file = File::create(&path).await.map_err(ErrorKind::IOError)?;

    log::info!("[sandbox] 开始下载... Downloading...");
    let mut res = reqwest::get(url).await.map_err(ErrorKind::RequestError)?;
    if !res.status().is_success() {
        return Err(ErrorKind::RespError(res.status()));
    }

    while let Some(chunk) = res.chunk().await.map_err(ErrorKind::RequestError)? {
        file.write_all(&chunk).await.map_err(ErrorKind::IOError)?;
    }

    file.sync_all().await.map_err(ErrorKind::IOError)?;
    log::info!("[sandbox] 下载完毕。 Download completed.");

    let path = sandbox::do_install(&path).map_err(ErrorKind::IOError)?;

    Ok(ComponentInfo::new(Version::Installed, Some(path)))
}

async fn install_yarn() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 Yarn... Start to install Yarn...");

    time::sleep(time::Duration::from_secs(20)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}

async fn install_pm2() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 PM2... Start to install PM2...");

    time::sleep(time::Duration::from_secs(5)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}

async fn install_hydro() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 Hydro... Start to install Hydro...");

    time::sleep(time::Duration::from_secs(5)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}
