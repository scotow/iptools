use anyhow::{bail, Context, Error as AnyError};
use ipnet::{IpNet, IpSubnets};
use itertools::Itertools;

use crate::{input::Input, source::Source};

pub fn process_batch(
    sources: Vec<Source>,
    prefix_len: u8,
    cidr: bool,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let input = Input::<IpNet>::Lazy(sources);
    if sort || unique {
        let mut nets = input
            .into_iter()
            .map(|net| process_single(net?, prefix_len))
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>()?;
        if sort {
            nets.sort();
        }
        if unique {
            nets = nets.into_iter().unique().collect();
        }
        if cidr {
            println!("{}", nets.iter().join("\n"));
        } else {
            println!("{}", nets.iter().map(IpNet::addr).join("\n"));
        }
    } else {
        for net in input {
            for subnet in process_single(net?, prefix_len)? {
                if cidr {
                    println!("{}", subnet);
                } else {
                    println!("{}", subnet.addr());
                }
            }
        }
    }

    Ok(())
}

fn process_single(net: IpNet, prefix_len: u8) -> Result<IpSubnets, AnyError> {
    if prefix_len < net.prefix_len() {
        bail!("prefix is shorter than original subnet")
    }
    net.subnets(prefix_len)
        .context("invalid subnet prefix length")
}
