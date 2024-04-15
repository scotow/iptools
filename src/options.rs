use clap::{Parser, Subcommand};

use crate::source::Source;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Options {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long = "input", global = true)]
    pub inputs: Vec<Source>,
    #[arg(short, long, global = true)]
    pub sort: bool,
    #[arg(short, long, alias = "uniq", global = true)]
    pub unique: bool,
    #[arg(global = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Cat,
    Info,
    Net {
        #[arg(short, long, alias = "prefix")]
        prefix_len: u8,
        #[arg(short = 'C', long)]
        cidr: bool,
    },
    Subnet {
        #[arg(short, long, alias = "prefix")]
        prefix_len: u8,
        #[arg(short = 'C', long)]
        cidr: bool,
    },
}
