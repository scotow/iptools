use std::{convert, hash::Hash, str::FromStr};

use anyhow::Error as AnyError;
use itertools::Itertools;

use crate::source::Source;

pub enum Input<T> {
    Memory(Vec<T>),
    Lazy(Vec<Source>),
}

impl<T> Input<T> {
    pub fn to_memory(self) -> Result<Input<T>, AnyError>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: Into<AnyError>,
    {
        match &self {
            Input::Memory(_values) => Ok(self),
            Input::Lazy(_) => Ok(Self::Memory(
                self.into_iter().collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }

    pub fn sort(self) -> Result<Input<T>, AnyError>
    where
        T: FromStr + Ord + 'static,
        <T as FromStr>::Err: Into<AnyError>,
    {
        let mut this = self.to_memory()?;
        match &mut this {
            Input::Memory(elems) => {
                elems.sort();
            }
            Input::Lazy(_) => unreachable!(),
        }
        Ok(this)
    }

    pub fn unique(self) -> Result<Input<T>, AnyError>
    where
        T: FromStr + Clone + Eq + Hash + 'static,
        <T as FromStr>::Err: Into<AnyError>,
    {
        match self.to_memory()? {
            Input::Memory(elems) => Ok(Self::Memory(elems.into_iter().unique().collect())),
            Input::Lazy(_) => unreachable!(),
        }
    }
}

impl<T> IntoIterator for Input<T>
where
    T: FromStr + 'static,
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
