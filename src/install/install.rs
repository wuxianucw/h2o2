use derive_more::{Constructor, Display};
use std::{
    result::Result as StdResult,
    sync::{Arc, Mutex},
};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
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

#[derive(Debug, Display)]
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
    pub yarn: bool,
}

pub type Result<T> = StdResult<T, Error>;

pub async fn install_nodejs(_states: Arc<Mutex<States>>) -> Result<()> {
    todo!();
}

pub async fn install_mongodb() -> Result<()> {
    todo!();
}

pub async fn install_minio() -> Result<()> {
    todo!();
}

pub async fn install_sandbox() -> Result<()> {
    todo!();
}

pub async fn install_yarn(_states: Arc<Mutex<States>>) -> Result<()> {
    todo!();
}

pub async fn install_pm2(_states: Arc<Mutex<States>>) -> Result<()> {
    todo!();
}

pub async fn install_hydro(_states: Arc<Mutex<States>>) -> Result<()> {
    todo!();
}
