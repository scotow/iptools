use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::source::Source;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Options {
    /// Path to the configuration file
    /// ({$PWD/.config,$HOME/.config,/etc}/iptools.toml if not specified).
    #[arg(short = 'C', long = "config", global = true)]
    pub configuration_path: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
    /// File path(s) to read from ('-' for stdin).
    #[arg(short, long = "input", global = true)]
    pub inputs: Vec<Source>,
    /// Sort results before displaying (allocation required).
    #[arg(short, long, global = true)]
    pub sort: bool,
    /// Remove duplicates before displaying (allocation required).
    #[arg(short, long, alias = "uniq", global = true)]
    pub unique: bool,
    /// Additional input(s) to process.
    #[arg(global = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Parse, validate and concatenate IP addresses or networks.
    Cat,
    /// Describe an IP address or network.
    Info {
        /// Disable column alignment padding.
        #[arg(short = 'P', long)]
        no_padding: bool,
    },
    /// Get the network address of an IP address.
    Net {
        /// Prefix length of desired network.
        #[arg(short, long, alias = "prefix")]
        prefix_len: u8,
        /// Display results using CIDR notation.
        #[arg(short, long)]
        cidr: bool,
    },
    /// List all the subnets of a network.
    Subnet {
        /// Prefix length of the desired subnets.
        #[arg(short, long, alias = "prefix")]
        prefix_len: u8,
        /// Display results using CIDR notation.
        #[arg(short, long)]
        cidr: bool,
    },
    /// List all the IP addresses belonging to a network.
    Hosts {
        /// Add network and broadcast address if available.
        #[arg(short, long)]
        all: bool,
    },
    /// Filter addresses and networks based on a query.
    Filter {
        /// Add network and broadcast address if available.
        #[arg(short, long)]
        query: String,
    },
    /// Get the first matching group of IP addresses or networks.
    Group {
        /// Exit on no group match.
        #[arg(short = 'f', long)]
        exit_no_match: bool,
    },
}
