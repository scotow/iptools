#![warn(unused_crate_dependencies)]

mod addr_or_net;
mod auto_net;
mod commands;
mod config;
mod group;
mod input;
mod options;
mod source;

use anyhow::bail;
use clap::Parser;

use crate::{
    config::Config,
    options::{Command, Options},
    source::Source,
};

fn main() -> Result<(), anyhow::Error> {
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
            commands::info::process_batch(sources, !no_padding, options.sort, options.unique)?;
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
        Command::Hosts { no_all } => {
            commands::hosts::process_batch(sources, !no_all, options.sort, options.unique)?;
        }
        Command::Merge => {
            commands::merge::process(sources, options.sort, options.unique)?;
        }
        Command::Filter { query } => commands::filter::process_batch(
            sources,
            query,
            Config::load(options.config_path)?,
            options.sort,
            options.unique,
        )?,
        Command::Group { exit_no_match } => {
            commands::group::process_batch(
                sources,
                Config::load(options.config_path)?,
                exit_no_match,
                options.sort,
                options.unique,
            )?;
        }
    }

    Ok(())
}
