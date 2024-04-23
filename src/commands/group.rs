use anyhow::{bail, Error as AnyError};
use itertools::Itertools;

use crate::{
    addr_or_net::AddrOrNet, configuration::Configuration, group, input::Input, source::Source,
};

pub fn process_batch(
    sources: Vec<Source>,
    mut configuration: Option<Configuration>,
    exit_no_match: bool,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let input = Input::<AddrOrNet>::Lazy(sources);
    if sort || unique {
        let mut groups = Vec::new();
        for value in input {
            let value = value?;
            match group::matching_groups(value, configuration.as_mut())?
                .next()
                .transpose()?
            {
                Some(group) => groups.push(group.to_owned()),
                None => {
                    if exit_no_match {
                        bail!("no group found for {}", value);
                    }
                }
            }
        }
        if sort {
            groups.sort();
        }
        if unique {
            groups = groups.into_iter().unique().collect();
        }
        println!("{}", groups.join("\n"));
    } else {
        for value in input {
            let value = value?;
            match group::matching_groups(value, configuration.as_mut())?
                .next()
                .transpose()?
            {
                Some(group) => println!("{group}"),
                None => {
                    if exit_no_match {
                        bail!("no group found for {}", value);
                    }
                }
            }
        }
    }

    Ok(())
}
