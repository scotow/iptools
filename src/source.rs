use std::{convert::Infallible, fs, io, io::Read, path::PathBuf, str::FromStr};

use anyhow::Error as AnyError;

#[derive(Clone, Debug)]
pub enum Source {
    File(PathBuf),
    Stdin,
    Arg(String),
}

impl Source {
    pub fn resolve(self) -> Result<String, AnyError> {
        Ok(match self {
            Source::File(path) => fs::read_to_string(path)?,
            Source::Stdin => {
                let mut buffer = String::new();
                io::stdin().lock().read_to_string(&mut buffer)?;
                buffer
            }
            Source::Arg(s) => s,
        }
        .trim()
        .to_owned())
    }
}

impl FromStr for Source {
    type Err = Infallible;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(if input == "-" {
            Self::Stdin
        } else {
            Self::File(PathBuf::from_str(input)?)
        })
    }
}
