use std::{
    convert::Infallible,
    fs::File,
    io::{self, BufRead, BufReader, Lines, StdinLock},
    path::PathBuf,
    str::FromStr,
};

use anyhow::Error as AnyError;

#[derive(Clone, Debug)]
pub enum Source {
    File(PathBuf),
    Stdin,
    Arg(String),
}

impl Source {
    pub fn into_iter(self) -> Result<IntoIter, AnyError> {
        Ok(match self {
            Source::File(path) => IntoIter::File(BufReader::new(File::open(&path)?).lines()),
            Source::Stdin => IntoIter::Stdin(io::stdin().lock().lines()),
            Source::Arg(arg) => IntoIter::Arg(Some(arg)),
        })
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

pub enum IntoIter {
    File(Lines<BufReader<File>>),
    Stdin(Lines<StdinLock<'static>>),
    Arg(Option<String>),
}

impl Iterator for IntoIter {
    type Item = Result<String, AnyError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::File(reader) => reader.next().map(|l| l.map_err(AnyError::from)),
            IntoIter::Stdin(lock) => lock.next().map(|l| l.map_err(AnyError::from)),
            IntoIter::Arg(arg) => arg.take().map(Ok),
        }
    }
}
