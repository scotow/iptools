#![warn(unused_crate_dependencies)]

mod addr_or_net;
mod auto_net;
mod commands;
mod configuration;
mod group;
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
            .map(Source::Arg)
            .chain(options.inputs)
            .collect()
    };

    match options.command {
        Command::Cat => commands::cat::process_batch(sources, options.sort, options.unique)?,
        Command::Info { no_padding } => {
            commands::info::process_batch(sources, !no_padding, options.sort, options.unique)?
        }
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
        Command::Group { exit_no_match } => {
            let configuration = Configuration::load(options.configuration_path)?;
            commands::group::process_batch(
                sources,
                configuration,
                exit_no_match,
                options.sort,
                options.unique,
            )?
        }
    }

    Ok(())
}
