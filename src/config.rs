use derive_more::IsVariant;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{fs, io};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// components infomation
    pub components: Components,

    pub profile: Profile,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Components {
    /// Node.js version
    pub nodejs: ComponentInfo,

    /// MongoDB version
    pub mongodb: ComponentInfo,

    /// MinIO version
    pub minio: ComponentInfo,

    /// sandbox version
    pub sandbox: ComponentInfo,

    /// Yarn version
    pub yarn: ComponentInfo,

    /// PM2 version
    pub pm2: ComponentInfo,

    /// Hydro version
    pub hydro: ComponentInfo,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ComponentInfo {
    pub version: Version,
    pub path: Option<String>,
}

impl ComponentInfo {
    pub fn to_show_format(&self) -> String {
        format!(
            "{}{}",
            self.version,
            self.path
                .as_ref()
                .map_or_else(String::new, |path| format!(" @ {}", path))
        )
    }

    pub fn is_installed(&self) -> bool {
        self.version.is_installed() || self.version.is_valid()
    }

    pub fn version(&self) -> Option<&semver::Version> {
        match &self.version {
            Version::Valid(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, IsVariant)]
pub enum Version {
    Unknown,
    Installed,
    Valid(semver::Version),
    Invalid(String),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Profile {}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file does not exist, please run `h2o2 detect` or `h2o2 install` first")]
    FileNotExist,

    #[error("Failed to read config file, consider running `h2o2 detect` to fix")]
    ReadError(#[source] io::Error),

    #[error("Failed to write config file")]
    WriteError(#[source] io::Error),

    #[error("Failed to deserialize config file, consider running `h2o2 detect` to fix")]
    DeserializeError(#[from] toml::de::Error),

    #[error("Failed to serialize config, please contact the developer")]
    SerializeError(#[from] toml::ser::Error),
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Unknown => "unknown".to_owned(),
                Self::Installed => "installed".to_owned(),
                Self::Valid(v) => v.to_string(),
                Self::Invalid(x) => x.to_owned(),
            }
        )
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&match self {
            Self::Unknown => "unknown".to_owned(),
            Self::Installed => "installed".to_owned(),
            Self::Valid(v) => v.to_string(),
            Self::Invalid(x) => x.to_owned(),
        })
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match &s[..] {
            "unknown" => Ok(Self::Unknown),
            "installed" => Ok(Self::Installed),
            text => Ok(match semver::Version::parse(text) {
                Ok(v) => Self::Valid(v),
                Err(_) => Self::Invalid(s),
            }),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let mut config_path = dirs::home_dir().expect("Failed to get home dir");
    config_path.push(".h2o2config");
    config_path
}

pub async fn load_config() -> Result<Config, ConfigError> {
    let config_path = get_config_path();

    if !Path::new(&config_path).is_file() {
        return Err(ConfigError::FileNotExist);
    }

    fs::read_to_string(config_path)
        .await
        .map_err(ConfigError::ReadError)
        .and_then(|text| toml::from_str(&text).map_err(ConfigError::DeserializeError))
}

pub async fn save_config(config: &Config) -> Result<(), ConfigError> {
    let config_path = get_config_path();
    fs::write(
        config_path,
        match toml::to_string(config) {
            Ok(text) => text,
            Err(e) => {
                return Err(ConfigError::SerializeError(e));
            }
        },
    )
    .await
    .map_err(ConfigError::WriteError)
}
