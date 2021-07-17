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

#[derive(Default)]
pub struct States {
    pub nodejs: State,
    pub mongodb: State,
    pub minio: State,
    pub sandbox: State,
    pub yarn: State,
    pub pm2: State,
}

#[derive(Debug, Display)]
pub enum State {
    Pending,
    Ready,
    Failed,
}

impl Default for State {
    fn default() -> State {
        Self::Pending
    }
}

impl States {
    pub fn borrow_by_com(&self, com: Com) -> Option<&State> {
        match com {
            Com::NodeJS => Some(&self.nodejs),
            Com::MongoDB => Some(&self.mongodb),
            Com::MinIO => Some(&self.minio),
            Com::Sandbox => Some(&self.sandbox),
            Com::Yarn => Some(&self.yarn),
            Com::PM2 => Some(&self.pm2),
            _ => None,
        }
    }

    pub fn borrow_mut_by_com(&mut self, com: Com) -> Option<&mut State> {
        match com {
            Com::NodeJS => Some(&mut self.nodejs),
            Com::MongoDB => Some(&mut self.mongodb),
            Com::MinIO => Some(&mut self.minio),
            Com::Sandbox => Some(&mut self.sandbox),
            Com::Yarn => Some(&mut self.yarn),
            Com::PM2 => Some(&mut self.pm2),
            _ => None,
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

pub async fn install(com: Com, states: &Arc<Mutex<States>>) -> Result<(Com, ComponentInfo)> {
    match com {
        // must await each, because `impl Future<Output = T>` is an opaque type
        Com::NodeJS => install_nodejs().await,
        Com::MongoDB => install_mongodb().await,
        Com::MinIO => install_minio().await,
        Com::Sandbox => install_sandbox().await,
        Com::Yarn => install_yarn(states.clone()).await,
        Com::PM2 => install_pm2(states.clone()).await,
        Com::Hydro => install_hydro(states.clone()).await,
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

async fn install_yarn(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_pm2(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}

async fn install_hydro(_states: Arc<Mutex<States>>) -> InstallResult<ComponentInfo> {
    todo!();
}
