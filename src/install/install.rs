use derive_more::{Constructor, Display, IsVariant};
use std::result::Result as StdResult;
use thiserror::Error as ThisError;
use tokio::{
    sync::broadcast::{error::RecvError, Receiver},
    time,
};

pub use crate::config::ComponentInfo;
use super::helper::*;

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
pub enum Signal {
    Ready(Com),
    Failed(Com),
}

macro_rules! wait_for_components {
    ($com:expr, $rx:expr, $($dep_com:expr),+ $(,)?) => {{
        let mut coms = vec![$($dep_com),+];
        while !coms.is_empty() {
            match $rx.recv().await.map_err(|e| Error::new($com, ErrorKind::RecvError(e)))? {
                Signal::Ready(com) => {
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

pub async fn install(com: Com, rx: Option<Receiver<Signal>>) -> Result<(Com, ComponentInfo)> {
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

    let (postfix, shasum256) = nodejs::get_binary_info();

    log::info!("{} {}", postfix, shasum256);

    time::sleep(time::Duration::from_secs(10)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}

async fn install_mongodb() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 MongoDB... Start to install MongoDB...");

    time::sleep(time::Duration::from_secs(20)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}

async fn install_minio() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 MinIO... Start to install MinIO...");

    time::sleep(time::Duration::from_secs(5)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
}

async fn install_sandbox() -> InstallResult<ComponentInfo> {
    log::info!("开始安装 sandbox... Start to install sandbox...");

    time::sleep(time::Duration::from_secs(3)).await;

    Err(ErrorKind::Other("not yet implemented".to_owned()))
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
