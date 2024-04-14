mod auto_net;
mod commands;
mod input;
mod options;
mod source;

use anyhow::{bail, Error as AnyError};
use clap::Parser;
use itertools::Itertools;

use crate::{
    auto_net::AutoNet,
    input::Input,
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
        Command::Info => {
            let mut input = Input::<AutoNet>::Lazy(sources);
            if options.sort {
                input = input.sort()?;
            }
            if options.unique {
                input = input.unique()?;
            }
            #[allow(unstable_name_collisions)]
            for value in input
                .into_iter()
                .map(|addr| addr.map(|addr| commands::info::process(addr.0)))
                .intersperse_with(|| Ok(String::new()))
            {
                match value {
                    Ok(output) => {
                        println!("{output}");
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        Command::Net { prefix_len, cidr } => {
            commands::net::process_batch(sources, prefix_len, cidr, options.sort, options.unique)?;
        }
    }

    Ok(())
}
