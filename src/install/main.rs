use anyhow::Result;
use clap::Clap;

use crate::{
    check_version,
    config::{self, Config, ConfigError},
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

    // find out the components that need installing, and then `try_join!` them together
    #[allow(unused_mut)]
    let mut com = &mut config.components;

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
    } else {
        todo!();
    }

    // MongoDB

    // MinIO

    // sandbox

    // components below depends on Node.js, should wait until it's ready

    // yarn

    // pm2

    // Hydro

    todo!();
}
