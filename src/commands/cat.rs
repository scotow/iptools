use anyhow::Error as AnyError;

use crate::{addr_or_net::AddrOrNet, input::Input, source::Source};

pub fn process_batch(sources: Vec<Source>, sort: bool, unique: bool) -> Result<(), AnyError> {
    let mut input = Input::<AddrOrNet>::Lazy(sources);
    if sort {
        input.sort()?;
    }
    if unique {
        input.unique()?;
    }

    for value in input {
        println!("{}", value?);
    }

    Ok(())
}
