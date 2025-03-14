use anyhow::Error as AnyError;

use crate::{addr_or_net::AddrOrNet, input::Input, source::Source};

pub fn process_batch(sources: Vec<Source>, sort: bool, unique: bool) -> Result<(), AnyError> {
    let mut input = Input::<AddrOrNet>::Lazy(sources);
    if unique {
        input.unique()?;
    }
    if sort {
        input.sort()?;
    }

    for value in input {
        println!("{}", value?);
    }

    Ok(())
}
