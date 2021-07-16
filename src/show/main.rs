use crate::config;
use anyhow::Result;
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "wuxianucw <i@ucw.moe>")]
pub struct Args {}

pub async fn main() -> Result<()> {
    let config = config::load_config().await?;
    println!("H2O2 show");
    println!();
    println!("目前 H2O2 配置文件中记录的组件状况如下：");
    println!("Components recorded in .h2o2config:");
    println!();
    show_components(&config.components);
    println!();
    println!("如果配置文件中记录的组件状况与实际情况不一致，请手动运行 `h2o2 detect` 来重新同步组件状况。");
    println!("If the components recorded is inconsistent with the actual situation, please run `h2o2 detect` to resync components.");
    Ok(())
}

pub fn show_components(com: &config::Components) {
    println!(" Node.js {}", com.nodejs.to_show_format());
    println!(" MongoDB {}", com.mongodb.to_show_format());
    println!(" MinIO   {}", com.minio.to_show_format());
    println!(" sandbox {}", com.sandbox.to_show_format());
    println!(" Yarn    {}", com.yarn.to_show_format());
    println!(" PM2     {}", com.pm2.to_show_format());
    println!(" Hydro   {}", com.hydro.to_show_format());
}
