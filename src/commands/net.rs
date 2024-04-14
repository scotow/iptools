use std::net::IpAddr;

use anyhow::Error as AnyError;
use ipnet::IpNet;
use itertools::Itertools;

use crate::{input::Input, source::Source};

pub fn process_batch(
    sources: Vec<Source>,
    prefix_len: u8,
    cidr: bool,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let input = Input::<IpAddr>::Lazy(sources);
    if sort || unique {
        let mut nets = input
            .into_iter()
            .map(|addr| process_single(addr?, prefix_len))
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
        for addr in input {
            let net = process_single(addr?, prefix_len)?;
            if cidr {
                println!("{}", net);
            } else {
                println!("{}", net.addr());
            }
        }
    }

    Ok(())
}

fn process_single(addr: IpAddr, prefix_len: u8) -> Result<IpNet, AnyError> {
    Ok(IpNet::new(addr, prefix_len)?.trunc())
}
