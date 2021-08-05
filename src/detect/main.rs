use anyhow::{Context, Result};
use clap::Clap;
use duct::cmd;
use semver::Version;
use std::{env, io::ErrorKind, path::Path};

use crate::{
    check_version,
    config::{self, Config, ConfigError},
    show,
    utils::debug_output,
};

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "wuxianucw <i@ucw.moe>")]
pub struct Args {
    /// 仅显示探测到的组件及版本，不更新配置文件
    /// Prints the components detected only without updating .h2o2config
    #[clap(short, long)]
    dry_run: bool,

    /// 不加载配置文件
    /// Runs without loading config
    #[clap(long)]
    no_config: bool,
}

pub async fn main(args: Args) -> Result<()> {
    let mut config = if args.no_config {
        log::info!("当前模式将不加载配置文件。 Skipped config loading.");
        // load config actually, because if not, sandbox config will lose
        match config::load_config().await {
            Ok(cfg) => {
                let mut config = Config::default();
                config.components.sandbox = cfg.components.sandbox;
                config
            }
            Err(_) => Config::default(),
        }
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

    let mut com = &mut config.components;
    let (mut nodejs_ok, mut yarn_ok) = (false, false);

    // detect Node.js
    log::info!("探测 Node.js... Detecting Node.js...");
    let executable = com.nodejs.path.as_deref().unwrap_or("node");
    // try to execute `node -v`
    match cmd!(executable, "-v")
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
    {
        Ok(output) => {
            let stdout =
                String::from_utf8(output.stdout.clone()).context("Failed to convert stdout")?;
            if output.status.success() {
                // try to parse version
                // stdout: v{version}
                let stdout = stdout.trim();
                if stdout.len() < "v?".len() {
                    log::error!(
                        "Node.js 的输出太短，疑似运行异常。 \
                        The output of Node.js is too short, and it seems to be running abnormally."
                    );
                    debug_output(&output);
                } else {
                    // skip the leading "v" and parse
                    match Version::parse(&stdout["v".len()..]) {
                        Ok(version) => {
                            log::info!("Found: Node.js {}", &version);
                            check_version!(nodejs, &version, warn);
                            com.nodejs.version = config::Version::Valid(version);
                            com.nodejs.path = Some(executable.to_owned());
                            nodejs_ok = true;
                        }
                        Err(e) => {
                            log::error!("解析版本号失败。 Failed to parse version.");
                            log::debug!("{:#?}", e);
                            debug_output(&output);
                        }
                    }
                }
            } else {
                log::error!(
                    "Node.js 异常退出，无法识别版本。 \
                    Node.js exited abnormally and the version could not be recognized. ({})",
                    &output.status,
                );
                debug_output(&output);
            }
        }
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                log::error!("未找到 Node.js。 Node.js is not found.");
            } else {
                log::error!(
                    "命令 `{} -v` 执行异常。 Failed to execute `{} -v`.",
                    executable,
                    executable
                );
                log::debug!("{:#?}", e);
            }
        }
    }

    // detect MongoDB
    log::info!("探测 MongoDB... Detecting MongoDB...");
    let executable = com.mongodb.path.as_deref().unwrap_or("mongod");
    // try to execute `mongod --version`
    match cmd!(executable, "--version")
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
    {
        Ok(output) => {
            let stdout =
                String::from_utf8(output.stdout.clone()).context("Failed to convert stdout")?;
            if output.status.success() {
                // try to parse version
                // stdout(first line): db version v{<version>}
                let stdout = stdout.lines().next().unwrap_or("");
                if stdout.len() < "db version v?".len() {
                    log::error!(
                        "MongoDB 的输出太短，疑似运行异常。 \
                        The output of MongoDB is too short, and it seems to be running abnormally."
                    );
                    debug_output(&output);
                } else {
                    // skip the leading "db version v" and parse
                    match Version::parse(&stdout["db version v".len()..]) {
                        Ok(version) => {
                            log::info!("Found: MongoDB {}", &version);
                            check_version!(mongodb, &version, warn);
                            com.mongodb.version = config::Version::Valid(version);
                            com.mongodb.path = Some(executable.to_owned());
                        }
                        Err(e) => {
                            log::error!("解析版本号失败。 Failed to parse version.");
                            log::debug!("{:#?}", e);
                            debug_output(&output);
                        }
                    }
                }
            } else {
                log::error!(
                    "MongoDB 异常退出，无法识别版本。 \
                    MongoDB exited abnormally and the version could not be recognized. ({})",
                    &output.status,
                );
                debug_output(&output);
            }
        }
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                log::error!("未找到 MongoDB。 MongoDB is not found.");
            } else {
                log::error!(
                    "命令 `{} --version` 执行异常。 Failed to execute `{} --version`.",
                    executable,
                    executable
                );
                log::debug!("{:#?}", e);
            }
        }
    }

    // detect MinIO
    log::info!("探测 MinIO... Detecting MinIO...");
    let executable = com.minio.path.as_deref().unwrap_or("minio");
    // try to execute `minio -v`
    match cmd!(executable, "-v")
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
    {
        Ok(output) => {
            let stdout =
                String::from_utf8(output.stdout.clone()).context("Failed to convert stdout")?;
            if output.status.success() {
                // simply check prefix
                // stdout: minio version {not a semver}
                // example: minio version RELEASE.2021-04-06T23-11-00Z
                let stdout = stdout.trim();
                if stdout.starts_with("minio version ") {
                    log::info!("Found: MinIO installed");
                    com.minio.version = config::Version::Installed;
                    com.minio.path = Some(executable.to_owned());
                } else {
                    log::error!(
                        "MinIO 的输出格式不正确，疑似运行异常。 \
                        The output format of MinIO is incorrect, and it seems to be running abnormally."
                    );
                    debug_output(&output);
                }
            } else {
                log::error!(
                    "MinIO 异常退出。 MinIO exited abnormally. ({})",
                    &output.status,
                );
                debug_output(&output);
            }
        }
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                log::error!("未找到 MinIO。 MinIO is not found.");
            } else {
                log::error!(
                    "命令 `{} -v` 执行异常。 Failed to execute `{} -v`.",
                    executable,
                    executable
                );
                log::debug!("{:#?}", e);
            }
        }
    }

    // detect sandbox
    log::info!("sandbox 无法探测，跳过。 Cannot detect sandbox, skipped.");

    // detect Yarn
    if nodejs_ok {
        log::info!("探测 Yarn... Detecting Yarn...");
        let executable =
            com.yarn
                .path
                .as_deref()
                .unwrap_or(if cfg!(windows) { "yarn.cmd" } else { "yarn" });
        // try to execute `yarn -v`
        match cmd!(executable, "-v")
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
        {
            Ok(output) => {
                let stdout =
                    String::from_utf8(output.stdout.clone()).context("Failed to convert stdout")?;
                if output.status.success() {
                    // try to parse version
                    // stdout: {version}
                    let stdout = stdout.trim();
                    match Version::parse(stdout) {
                        Ok(version) => {
                            log::info!("Found: Yarn {}", &version);
                            com.yarn.version = config::Version::Valid(version);
                            com.yarn.path = Some(executable.to_owned());
                            yarn_ok = true;
                        }
                        Err(e) => {
                            log::error!("解析版本号失败。 Failed to parse version.");
                            log::debug!("{:#?}", e);
                            debug_output(&output);
                        }
                    }
                } else {
                    log::error!(
                        "Yarn 异常退出，无法识别版本。 \
                        Yarn exited abnormally and the version could not be recognized. ({})",
                        &output.status,
                    );
                    debug_output(&output);
                }
            }
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    log::error!("未找到 Yarn。 Yarn is not found.");
                } else {
                    log::error!(
                        "命令 `{} -v` 执行异常。 Failed to execute `{} -v`.",
                        executable,
                        executable
                    );
                    log::debug!("{:#?}", e);
                }
            }
        }
    } else {
        log::warn!(
            "未找到 Node.js，跳过 Yarn（依赖 Node.js）。 \
            Skip Yarn (which depends on Node.js) due to Node.js not found."
        );
    }

    // detect PM2
    if nodejs_ok {
        log::info!("探测 PM2... Detecting PM2...");
        let executable =
            com.pm2
                .path
                .as_deref()
                .unwrap_or(if cfg!(windows) { "pm2.cmd" } else { "pm2" });
        // try to execute `pm2 -v -s --no-daemon`
        match cmd!(executable, "-v", "-s", "--no-daemon")
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
        {
            Ok(output) => {
                let stdout =
                    String::from_utf8(output.stdout.clone()).context("Failed to convert stdout")?;
                if output.status.success() {
                    // try to parse version
                    // stdout: {version}
                    let stdout = stdout.trim();
                    match Version::parse(stdout) {
                        Ok(version) => {
                            log::info!("Found: PM2 {}", &version);
                            com.pm2.version = config::Version::Valid(version);
                            com.pm2.path = Some(executable.to_owned());
                        }
                        Err(e) => {
                            log::error!("解析版本号失败。 Failed to parse version.");
                            log::debug!("{:#?}", e);
                            debug_output(&output);
                        }
                    }
                } else {
                    log::error!(
                        "PM2 异常退出，无法识别版本。 \
                        PM2 exited abnormally and the version could not be recognized. ({})",
                        &output.status,
                    );
                    debug_output(&output);
                }
            }
            Err(e) => {
                if let ErrorKind::NotFound = e.kind() {
                    log::error!("未找到 PM2。 PM2 is not found.");
                } else {
                    log::error!(
                        "命令 `{} -v -s --no-daemon` 执行异常。 Failed to execute `{} -v -s --no-daemon`.",
                        executable,
                        executable
                    );
                    log::debug!("{:#?}", e);
                }
            }
        }
    } else {
        log::warn!(
            "未找到 Node.js，跳过 PM2（依赖 Node.js）。 \
            Skip PM2 (which depends on Node.js) due to Node.js not found."
        );
    }

    // detect Hydro
    if yarn_ok {
        log::info!("探测 Hydro... Detecting Hydro...");
        let yarn_global_dir;
        let path = match com.hydro.path.as_deref() {
            Some(path) => path,
            None => {
                let yarn = com
                    .yarn
                    .path
                    .as_ref()
                    .expect("Yarn should be OK, but its `path` is `None`");
                yarn_global_dir = cmd!(yarn, "global", "dir")
                    .unchecked()
                    .read()
                    .with_context(|| {
                        format!("Failed to get the result of `{} global dir`", yarn)
                    })?;
                &yarn_global_dir
            }
        };
        // Note: `path` may not exist
        if Path::new(path).is_dir() {
            // try to execute some magic command
            let current_dir = env::current_dir().context("Current dir is not available")?;
            env::set_current_dir(path).context("Failed to change working dir")?;
            let node = com
                .nodejs
                .path
                .as_deref()
                .expect("Node.js should be OK, but its `path` is `None`");
            match cmd!(
                node,
                "-e",
                "console.log(require('hydrooj/package.json').version)"
            )
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
            {
                Ok(output) => {
                    let stdout = String::from_utf8(output.stdout.clone())
                        .context("Failed to convert stdout")?;
                    if output.status.success() {
                        // try to parse version
                        // stdout: {version}
                        let stdout = stdout.trim();
                        match Version::parse(stdout) {
                            Ok(version) => {
                                log::info!("Found: Hydro {}", &version);
                                com.hydro.version = config::Version::Valid(version);
                                com.hydro.path = Some(path.to_owned());
                            }
                            Err(e) => {
                                log::error!("解析版本号失败。 Failed to parse version.");
                                log::debug!("{:#?}", e);
                                debug_output(&output);
                            }
                        }
                    } else {
                        log::error!("未找到 Hydro。 Hydro is not found.");
                        debug_output(&output);
                    }
                }
                Err(e) => {
                    if let ErrorKind::NotFound = e.kind() {
                        log::error!("未找到 Hydro。 Hydro is not found.");
                    } else {
                        log::error!(
                            "命令 `{} -e <...>` 执行异常。 Failed to execute `{} -e <...>`.",
                            node,
                            node
                        );
                        log::debug!("{:#?}", e);
                    }
                }
            }
            env::set_current_dir(current_dir).context("Failed to change working dir")?;
        } else {
            log::error!("未找到 Hydro。 Hydro is not found.");
        }
    } else {
        log::warn!(
            "未找到 Yarn，跳过 Hydro（依赖 Yarn）。 \
            Skip Hydro (which depends on Yarn) due to Yarn not found."
        );
    }

    log::info!("结果如下： Result:");
    show::show_components(com);
    if args.dry_run {
        return Ok(());
    }

    log::info!("将写入配置文件... Saving config...");
    config::save_config(&config).await?;
    log::info!("配置已成功保存。 Config saved successfully.");

    Ok(())
}
