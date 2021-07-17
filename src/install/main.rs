use anyhow::Result;
use clap::Clap;
use futures::{stream::FuturesUnordered, StreamExt};
use std::sync::{Arc, Mutex};

use crate::{
    check_version,
    config::{self, Config, ConfigError},
    install::{install, Com, States},
};

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "wuxianucw <i@ucw.moe>")]
pub struct Args {
    /// 不加载配置文件
    /// Runs without loading config
    #[clap(long)]
    no_config: bool,
}

pub async fn main(args: Args) -> Result<()> {
    let mut config = if args.no_config {
        log::info!("当前模式将不加载配置文件。 Skipped config loading.");
        // always reinstall sandbox
        Config::default()
    } else {
        match config::load_config().await {
            Ok(config) => {
                log::info!("已成功加载配置。 Config loaded successfully.");
                config
            }
            Err(e) => {
                match e {
                    ConfigError::FileNotExist => {
                        log::info!("配置文件不存在，开始初始化。 Config file does not exist, start initialization.");
                    }
                    e => {
                        log::error!("加载配置失败！准备尝试重新初始化。 Failed to load config! Try to reinitialize.");
                        log::debug!("{:#?}", e);
                    }
                };
                Config::default()
            }
        }
    };

    // find out the components that need installing, and then execute them together
    #[allow(unused_mut)]
    let mut com = &mut config.components;
    let mut tasks = Vec::new();
    let states = Arc::new(Mutex::new(States::new(
        false, false, false, false, false, false,
    )));

    // Node.js
    log::info!("检查 Node.js... Checking Node.js...");
    if com.nodejs.is_installed() {
        log::info!("Node.js 已安装，不执行任何操作。 Node.js is already installed, skip.");
        let version = com
            .nodejs
            .version()
            .expect("Node.js should have a version if installed");
        check_version!(nodejs, version, warn);
        log::info!(
            "若需要 H2O2 安装一个推荐版本的 Node.js，请删除系统中已存在的版本并重新运行 H2O2。 \
            If you need H2O2 to install a recommended version of Node.js, \
            please delete the existing version in the system and run H2O2 again."
        );
        states.lock().expect("Failed to lock states").nodejs = true;
    } else {
        tasks.push(Com::NodeJS);
    }

    // MongoDB
    if com.mongodb.is_installed() {
        log::info!("MongoDB 已安装，不执行任何操作。 MongoDB is already installed, skip.");
        let version = com
            .mongodb
            .version()
            .expect("MongoDB should have a version if installed");
        check_version!(mongodb, version, warn);
        states.lock().expect("Failed to lock states").mongodb = true;
    } else {
        tasks.push(Com::MongoDB);
    }

    // MinIO
    if com.minio.is_installed() {
        log::info!("MinIO 已安装，不执行任何操作。 MinIO is already installed, skip.");
        states.lock().expect("Failed to lock states").minio = true;
    } else {
        tasks.push(Com::MinIO);
    }

    // sandbox
    if com.sandbox.is_installed() {
        log::info!("sandbox 已安装，不执行任何操作。 sandbox is already installed, skip.");
        states.lock().expect("Failed to lock states").sandbox = true;
    } else {
        tasks.push(Com::Sandbox);
    }

    // components below depends on Node.js, should wait until it's ready
    // this depends on shared states

    // yarn
    if com.yarn.is_installed() {
        log::info!("Yarn 已安装，不执行任何操作。 Yarn is already installed, skip.");
        states.lock().expect("Failed to lock states").yarn = true;
    } else {
        tasks.push(Com::Yarn);
    }

    // pm2
    if com.pm2.is_installed() {
        log::info!("PM2 已安装，不执行任何操作。 PM2 is already installed, skip.");
        states.lock().expect("Failed to lock states").pm2 = true;
    } else {
        tasks.push(Com::PM2);
    }

    // Hydro
    if com.hydro.is_installed() {
        log::info!("Hydro 已安装，不执行任何操作。 Hydro is already installed, skip.");
        log::info!(
            "若需要检查更新 Hydro，请运行 `h2o2 check`。 \
            If you need to check and update Hydro, please run `h2o2 check`."
        );
    } else {
        tasks.push(Com::Hydro);
    }

    let mut tasks = tasks
        .into_iter()
        .map(|com| install(com, states.clone()))
        .collect::<FuturesUnordered<_>>();

    while let Some(res) = tasks.next().await {
        match res {
            Ok((com_id, com_info)) => {
                log::info!("OK: {} {}", &com_id, com_info.to_show_format());
                *com.borrow_mut_by_com(com_id) = com_info;
            }
            Err(e) => {
                log::error!("安装 {} 失败！", e.com);
                log::error!("{}", e);
            }
        }
    }

    todo!();
}
