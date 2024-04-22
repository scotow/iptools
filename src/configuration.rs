use std::{env, fs, path::PathBuf, process::Command};

use anyhow::{Context, Error as AnyError};
use config_finder::ConfigDirs;
use serde::Deserialize;

use crate::auto_net::AutoNet;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub groups: Option<Vec<Group>>,
}

impl Configuration {
    pub fn load(path: Option<PathBuf>) -> Result<Option<Self>, AnyError> {
        match path {
            Some(path) => Ok(toml::from_str(&fs::read_to_string(path)?)?),
            None => {
                let mut configs = ConfigDirs::empty();
                for config in configs
                    .add_current_dir()?
                    .add_platform_config_dir()
                    .add_root_etc()
                    .search("", env!("CARGO_PKG_NAME"), "toml")
                {
                    if config.path().exists() {
                        let content = fs::read_to_string(config.path())?;
                        return Ok(toml::from_str(&content)?);
                    }
                }
                Ok(None)
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Group {
    pub name: String,
    #[serde(flatten)]
    pub source: GroupSource,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GroupSource {
    Raw {
        nets: Vec<AutoNet>,
    },
    File {
        file: PathBuf,
    },
    Command {
        command: String,
        shell: Option<String>,
    },
}

impl GroupSource {
    pub fn load(&mut self) -> Result<&[AutoNet], AnyError> {
        match self {
            GroupSource::Raw { nets } => Ok(nets),
            GroupSource::File { file: path } => {
                let nets = fs::read_to_string(path)?
                    .trim()
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(str::parse)
                    .collect::<Result<Vec<_>, _>>()
                    .context("invalid group file content")?;
                *self = Self::Raw { nets };
                self.load()
            }
            GroupSource::Command { command, shell } => {
                let shell = shell
                    .clone()
                    .or_else(|| env::var("SHELL").ok())
                    .unwrap_or_else(|| "sh".to_owned());
                let output = String::from_utf8(
                    Command::new(&shell)
                        .args(["-c", &command])
                        .output()
                        .context("group command failure")?
                        .stdout,
                )?;
                let nets = output
                    .trim()
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(str::parse)
                    .collect::<Result<Vec<_>, _>>()
                    .context("invalid group command output")?;
                *self = Self::Raw { nets };
                self.load()
            }
        }
    }
}
