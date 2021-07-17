use derive_more::{Constructor, Display};
use std::{
    result::Result as StdResult,
    sync::{Arc, Mutex},
};
use thiserror::Error as ThisError;

pub use crate::config::ComponentInfo;

#[derive(ThisError, Debug, Constructor)]
#[error("Failed to install {com}: {kind}")]
pub struct Error {
    pub com: Com,
    pub kind: ErrorKind,
}

#[derive(Debug, Display)]
pub enum ErrorKind {
    // TODO: more error kind
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

#[derive(Constructor)]
pub struct States {
    pub nodejs: bool,
    pub mongodb: bool,
    pub minio: bool,
    pub sandbox: bool,
    pub yarn: bool,
    pub pm2: bool,
}

pub type Result<T> = StdResult<T, Error>;

pub async fn install(com: Com, states: Arc<Mutex<States>>) -> Result<(Com, ComponentInfo)> {
    match com {
        // must await each, because `impl Future<Output = T>` is an opaque type
        Com::NodeJS => install_nodejs(states).await,
        Com::MongoDB => install_mongodb(states).await,
        Com::MinIO => install_minio(states).await,
        Com::Sandbox => install_sandbox(states).await,
        Com::Yarn => install_yarn(states).await,
        Com::PM2 => install_pm2(states).await,
        Com::Hydro => install_hydro(states).await,
    }
    .map(|ok| (com, ok))
    .map_err(|e| Error::new(com, e))
}

type InstallResult<T> = StdResult<T, ErrorKind>;

async fn install_nodejs(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_mongodb(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_minio(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_sandbox(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_yarn(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_pm2(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_hydro(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}
