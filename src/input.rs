use std::{
    collections::HashSet,
    convert,
    hash::{Hash, RandomState},
    mem,
    str::FromStr,
};

use anyhow::Error as AnyError;
use itertools::Itertools;

use crate::source::Source;

pub enum Input<T> {
    Memory(Vec<T>),
    Lazy(Vec<Source>),
}

impl<T> Input<T> {
    pub fn to_memory(&mut self) -> Result<(), AnyError>
    where
        T: FromStr,
        <T as FromStr>::Err: Into<AnyError>,
    {
        match self {
            Self::Memory(_values) => Ok(()),
            Self::Lazy(sources) => {
                *self = Self::Memory(
                    Self::Lazy(mem::take(sources))
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()?,
                );
                Ok(())
            }
        }
    }

    pub fn sort(&mut self) -> Result<(), AnyError>
    where
        T: FromStr + Ord,
        <T as FromStr>::Err: Into<AnyError>,
    {
        self.to_memory()?;
        match self {
            Input::Memory(elems) => {
                elems.sort();
            }
            Input::Lazy(_) => unreachable!(),
        }
        Ok(())
    }

    pub fn unique(&mut self) -> Result<(), AnyError>
    where
        T: FromStr + Eq + Hash,
        <T as FromStr>::Err: Into<AnyError>,
    {
        self.to_memory()?;
        match self {
            Input::Memory(elems) => {
                *elems = HashSet::<_, RandomState>::from_iter(mem::take(elems))
                    .into_iter()
                    .collect()
            }
            Input::Lazy(_) => unreachable!(),
        }
        Ok(())
    }
}

impl<T> IntoIterator for Input<T>
where
    T: FromStr,
    <T as FromStr>::Err: Into<AnyError>,
{
    type Item = Result<T, AnyError>;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Input::Memory(values) => IntoIter::Memory(values.into_iter()),
            Input::Lazy(sources) => IntoIter::Lazy(Box::new({
                sources
                    .into_iter()
                    .map(|s| s.into_iter())
                    .flatten_ok()
                    .filter(|l| match l {
                        Ok(Ok(l)) => !l.trim().is_empty(),
                        _ => true,
                    })
                    .map(|source| {
                        source.and_then(convert::identity).and_then(|l| {
                            T::from_str(l.trim()).map_err(|err| {
                                err.into()
                                    .context(format!("invalid address or network: {l}"))
                            })
                        })
                    })
            })),
        }
    }
}

pub enum IntoIter<T> {
    Memory(std::vec::IntoIter<T>),
    Lazy(Box<dyn Iterator<Item = Result<T, AnyError>>>),
}

impl<T> Iterator for IntoIter<T> {
    type Item = Result<T, AnyError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Memory(iter) => iter.next().map(Ok),
            IntoIter::Lazy(iter) => iter.next(),
        }
    }
}
