use anyhow::{bail, Context, Result};
use clap::Clap;
use futures::{stream::FuturesUnordered, StreamExt};
use std::path::Path;
use tokio::{fs, sync::broadcast};

use crate::{
    check_version,
    config::{self, Config, ConfigError},
    install::{install, Com, ComponentInfo, Signal},
    maybe_cmd,
};

macro_rules! run {
    ($($arg:expr),*) => {
        ::duct::cmd!($($arg),*)
            .stdout_capture()
            .stderr_capture()
            .run()
    };
}

macro_rules! expect {
    ($output:expr => valid => semver) => {
        $output.map_err(|_| ()).and_then(|output| {
            let stdout = String::from_utf8(output.stdout).map_err(|_| ())?;
            let stdout = stdout.lines().next().unwrap_or("").trim();
            ::semver::Version::parse(&stdout).map_err(|_| ())
        })
    };
    ($output:expr => $prefix:expr => semver) => {
        $output.map_err(|_| ()).and_then(|output| {
            let stdout = String::from_utf8(output.stdout).map_err(|_| ())?;
            let stdout = stdout.lines().next().unwrap_or("").trim();
            if stdout.len() <= $prefix.len() {
                return Err(());
            }
            ::semver::Version::parse(&stdout[$prefix.len()..]).map_err(|_| ())
        })
    };
    ($output:expr => valid) => {
        expect!($output => valid => semver).map($crate::config::Version::Valid)
    };
    ($output:expr => $prefix:expr) => {
        expect!($output => $prefix => semver).map($crate::config::Version::Valid)
    };
    ($output:expr => starts with $prefix:expr) => {
        $output.map_err(|_| ()).and_then(|output| {
            let stdout = String::from_utf8(output.stdout).map_err(|_| ())?;
            let stdout = stdout.trim();
            if stdout.starts_with($prefix) {
                Ok($crate::config::Version::Installed)
            } else {
                Err(())
            }
        })
    };
}

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "wuxianucw <i@ucw.moe>")]
pub struct Args {
    /// 不加载配置文件
    /// Runs without loading config
    #[clap(long)]
    no_config: bool,
}

pub async fn main(args: Args) -> Result<()> {
    // FIXME: support macos
    if cfg!(target_os = "macos") {
        bail!("Platform is not supported");
    }

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

    let com_path = config::get_com_path();
    if !Path::new(&com_path).is_dir() {
        fs::create_dir(&com_path)
            .await
            .context("创建目录失败！ Failed to create directory!")?;
    }

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
        let _ = tx.send(Signal::Ready(Com::Yarn, &com.yarn)); // Note: `tx.send()` may fail if there is no receiver
    } else if let Ok(v) = expect!(
        run!(maybe_cmd!("yarn"), "-v") => valid
    ) {
        log::info!("Yarn 已安装，不执行任何操作。 Yarn is already installed, skip.");
        com.yarn.path = Some(maybe_cmd!("yarn").to_owned());
        com.yarn.version = v;
        let _ = tx.send(Signal::Ready(Com::Yarn, &com.yarn));
    } else {
        tasks.push((Com::Yarn, Some(tx.subscribe())));
    }

    // PM2
    if com.pm2.is_installed() {
        log::info!("PM2 已安装，不执行任何操作。 PM2 is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::PM2, &com.pm2));
    } else if let Ok(v) = expect!(
        run!(maybe_cmd!("pm2"), "-v", "-s", "--no-daemon") => valid
    ) {
        log::info!("PM2 已安装，不执行任何操作。 PM2 is already installed, skip.");
        com.pm2.path = Some(maybe_cmd!("pm2").to_owned());
        com.pm2.version = v;
        let _ = tx.send(Signal::Ready(Com::PM2, &com.pm2));
    } else {
        tasks.push((Com::PM2, Some(tx.subscribe())));
    }

    // Node.js
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
        let _ = tx.send(Signal::Ready(Com::NodeJS, &com.nodejs));
    } else if let Ok(v) = expect!(
        run!("node", "-v") => "v" => semver
    ) {
        log::info!("Node.js 已安装，不执行任何操作。 Node.js is already installed, skip.");
        check_version!(nodejs, &v, warn);
        log::info!(
            "若需要 H2O2 安装一个推荐版本的 Node.js，请删除系统中已存在的版本并重新运行 H2O2。 \
            If you need H2O2 to install a recommended version of Node.js, \
            please delete the existing version in the system and run H2O2 again."
        );
        com.nodejs.path = None;
        com.nodejs.version = config::Version::Valid(v);
        let _ = tx.send(Signal::Ready(Com::NodeJS, &com.nodejs));
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
        let _ = tx.send(Signal::Ready(Com::MongoDB, &com.mongodb));
    } else if let Ok(v) = expect!(
        run!("mongod", "--version") => "db version v" => semver
    ) {
        log::info!("MongoDB 已安装，不执行任何操作。 MongoDB is already installed, skip.");
        check_version!(mongodb, &v, warn);
        com.mongodb.path = Some("mongod".to_owned());
        com.mongodb.version = config::Version::Valid(v);
        let _ = tx.send(Signal::Ready(Com::MongoDB, &com.mongodb));
    } else {
        tasks.push((Com::MongoDB, None));
    }

    // MinIO
    if com.minio.is_installed() {
        log::info!("MinIO 已安装，不执行任何操作。 MinIO is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::MinIO, &com.minio));
    } else if let Ok(v) = expect!(
        run!("minio", "-v") => starts with "minio version "
    ) {
        log::info!("MinIO 已安装，不执行任何操作。 MinIO is already installed, skip.");
        com.minio.path = Some("minio".to_owned());
        com.minio.version = v;
        let _ = tx.send(Signal::Ready(Com::MinIO, &com.minio));
    } else {
        tasks.push((Com::MinIO, None));
    }

    // sandbox
    if com.sandbox.is_installed() {
        log::info!("sandbox 已安装，不执行任何操作。 sandbox is already installed, skip.");
        let _ = tx.send(Signal::Ready(Com::Sandbox, &com.sandbox));
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
                let info = com.borrow_by_com(com_id);
                // Hack: *info = com_info;
                // For each time, we only modify a different part of `com`.
                // This is obviously safe, but rustc can't understand it.
                // `Mutex` is also an option, but it is costly.
                let info_ptr = info as *const ComponentInfo;
                unsafe {
                    *(info_ptr as *mut ComponentInfo) = com_info;
                }
                let _ = tx.send(Signal::Ready(com_id, info));
            }
            Err(e) => {
                log::error!("安装 {} 失败！", e.com); // English is no need because the error message is already in English
                log::error!("{}", e);
                let _ = tx.send(Signal::Failed(e.com));
            }
        }
    }

    if cfg!(unix) {
        // FIXME: exec $SHELL
        log::warn!(
            "请手动执行 `source ~/.profile` 来应用更改。 \
            Please execute `source ~/.profile` manually to apply changes."
        );
    }

    todo!();
}
