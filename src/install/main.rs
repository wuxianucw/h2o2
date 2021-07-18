use anyhow::Result;
use clap::Clap;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::broadcast;

use crate::{
    check_version,
    config::{self, Config, ConfigError},
    install::{install, Com, Signal},
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
    let com = &mut config.components;
    let mut tasks = Vec::new();
    let (tx, _) = broadcast::channel(16);

    // Hack: the order is vital, because we must make sure that `tx.subcribe()` is called
    // before `tx.send()`

    // Hydro
    if com.hydro.is_installed() {
        log::info!("Hydro 已安装，不执行任何操作。 Hydro is already installed, skip.");
        log::info!(
            "若需要检查更新 Hydro，请运行 `h2o2 check`。 \
            If you need to check and update Hydro, please run `h2o2 check`."
        );
    } else {
        tasks.push((Com::Hydro, Some(tx.subscribe())));
    }

    // Yarn
    if com.yarn.is_installed() {
        log::info!("Yarn 已安装，不执行任何操作。 Yarn is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::Yarn)); // Note: `tx.send()` may fail if there is no receiver
    } else {
        tasks.push((Com::Yarn, Some(tx.subscribe())));
    }

    // PM2
    if com.pm2.is_installed() {
        log::info!("PM2 已安装，不执行任何操作。 PM2 is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::PM2));
    } else {
        tasks.push((Com::PM2, Some(tx.subscribe())));
    }

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
        let _ = tx.send(Signal::Ready(Com::NodeJS));
    } else {
        tasks.push((Com::NodeJS, None));
    }

    // MongoDB
    if com.mongodb.is_installed() {
        log::info!("MongoDB 已安装，不执行任何操作。 MongoDB is already installed, skip.");
        let version = com
            .mongodb
            .version()
            .expect("MongoDB should have a version if installed");
        check_version!(mongodb, version, warn);
        let _ = tx.send(Signal::Ready(Com::MongoDB));
    } else {
        tasks.push((Com::MongoDB, None));
    }

    // MinIO
    if com.minio.is_installed() {
        log::info!("MinIO 已安装，不执行任何操作。 MinIO is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::MinIO));
    } else {
        tasks.push((Com::MinIO, None));
    }

    // sandbox
    if com.sandbox.is_installed() {
        log::info!("sandbox 已安装，不执行任何操作。 sandbox is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::Sandbox));
    } else {
        tasks.push((Com::Sandbox, None));
    }

    let mut tasks = tasks
        .into_iter()
        .map(|(com, rx)| install(com, rx))
        .collect::<FuturesUnordered<_>>();

    while let Some(res) = tasks.next().await {
        match res {
            Ok((com_id, com_info)) => {
                log::info!("OK: {} {}", &com_id, com_info.to_show_format());
                *com.borrow_mut_by_com(com_id) = com_info;
                let _ = tx.send(Signal::Ready(com_id));
            }
            Err(e) => {
                log::error!("安装 {} 失败！", e.com); // English is no need because the error message is already in English
                log::error!("{}", e);
                let _ = tx.send(Signal::Failed(e.com));
            }
        }
    }

    todo!();
}
