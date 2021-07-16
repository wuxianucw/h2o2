use anyhow::Result;
use clap::{AppSettings, Clap};

/// H2O2 (a.k.a. hydrogen peroxide): Another powerful tool for Hydro(hydro.js.org)
#[derive(Clap, Debug)]
#[clap(name = "H2O2")]
#[clap(version = "0.1.0", author = "wuxianucw <i@ucw.moe>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    /// 显示配置文件中记录的组件信息
    /// Prints the components recorded in .h2o2config
    #[clap(setting = AppSettings::ColoredHelp)]
    Show(h2o2::show::Args),

    /// 检查组件更新
    /// Checks for component updates
    #[clap(setting = AppSettings::ColoredHelp)]
    Check,

    /// 更新组件
    /// Updates components
    #[clap(setting = AppSettings::ColoredHelp)]
    Update(h2o2::update::Args),

    /// 探测已安装的组件并更新配置文件
    /// Detects the components installed and updates config
    #[clap(setting = AppSettings::ColoredHelp)]
    Detect(h2o2::detect::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    h2o2::log::init();
    let args = Args::parse();

    if cfg!(target_arch = "x86") {
        log::warn!(
            "x86 架构不受支持，Hydro 将无法正常工作，请考虑使用 x86_64。 \
            The x86 architecture is not supported, Hydro will not work properly, please consider using x86_64."
        );
    }

    match args.subcmd {
        SubCommand::Show(_) => h2o2::show::main().await?,
        SubCommand::Check => h2o2::check::main().await?,
        SubCommand::Update(args) => h2o2::update::main(args).await?,
        SubCommand::Detect(args) => h2o2::detect::main(args).await?,
    }

    Ok(())
}
