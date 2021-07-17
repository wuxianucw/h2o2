use derive_more::{Constructor, Display, IsVariant};
use std::result::Result as StdResult;
use thiserror::Error as ThisError;
use tokio::sync::broadcast::{error::RecvError, Receiver};

pub use crate::config::ComponentInfo;

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

    #[display(fmt = "{}", _0)]
    Other(String),
}

#[derive(Debug, Display, Copy, Clone)]
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

pub type Result<T> = StdResult<T, Error>;

pub async fn install(com: Com, rx: Option<Receiver<Signal>>) -> Result<(Com, ComponentInfo)> {
    match com {
        // must await each, because `impl Future<Output = T>` is an opaque type
        Com::NodeJS => install_nodejs().await,
        Com::MongoDB => install_mongodb().await,
        Com::MinIO => install_minio().await,
        Com::Sandbox => install_sandbox().await,
        Com::Yarn => install_yarn(rx.expect("Receiver cannot be `None`")).await,
        Com::PM2 => install_pm2(rx.expect("Receiver cannot be `None`")).await,
        Com::Hydro => install_hydro(rx.expect("Receiver cannot be `None`")).await,
    }
    .map(|ok| (com, ok))
    .map_err(|e| Error::new(com, e))
}

type InstallResult<T> = StdResult<T, ErrorKind>;

async fn install_nodejs() -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_mongodb() -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_minio() -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_sandbox() -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_yarn(mut rx: Receiver<Signal>) -> InstallResult<ComponentInfo> {
    loop {
        match rx.recv().await.map_err(ErrorKind::RecvError)? {
            Signal::Ready(com) => {
                if matches!(com, Com::NodeJS) {
                    break;
                }
            }
            Signal::Failed(com) => {
                if matches!(com, Com::NodeJS) {
                    return Err(ErrorKind::DependencyError(com));
                }
            }
        }
    }
    todo!();
}

async fn install_pm2(_rx: Receiver<Signal>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_hydro(_rx: Receiver<Signal>) -> InstallResult<ComponentInfo> {
    todo!();
}
