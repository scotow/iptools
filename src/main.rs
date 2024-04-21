#![warn(unused_crate_dependencies)]

mod addr_or_net;
mod auto_net;
mod commands;
mod configuration;
mod input;
mod options;
mod source;

use anyhow::{bail, Error as AnyError};
use clap::Parser;

use crate::{
    configuration::Configuration,
    options::{Command, Options},
    source::Source,
};

fn main() -> Result<(), AnyError> {
    let options = Options::parse();

    let sources = if options.inputs.is_empty() && options.args.is_empty() {
        vec![Source::Stdin]
    } else {
        if options
            .inputs
            .iter()
            .filter(|input| matches!(input, Source::Stdin))
            .count()
            >= 2
        {
            bail!(r#"multiple "-" file path specified"#);
        }
        options
            .args
            .into_iter()
            .map(|a| Source::Arg(a))
            .chain(options.inputs)
            .collect()
    };

    match options.command {
        Command::Cat => commands::cat::process_batch(sources, options.sort, options.unique)?,
        Command::Info => commands::info::process_batch(sources, options.sort, options.unique)?,
        Command::Net { prefix_len, cidr } => {
            commands::net::process_batch(sources, prefix_len, cidr, options.sort, options.unique)?;
        }
        Command::Subnet { prefix_len, cidr } => {
            commands::subnet::process_batch(
                sources,
                prefix_len,
                cidr,
                options.sort,
                options.unique,
            )?;
        }
        Command::Hosts { all } => {
            commands::hosts::process_batch(sources, all, options.sort, options.unique)?
        }
        Command::Filter { query } => {
            let configuration = Configuration::load(options.configuration_path)?;
            commands::filter::process_batch(
                sources,
                query,
                configuration,
                options.sort,
                options.unique,
            )?
        }
    }

    Ok(())
}
