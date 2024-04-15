use std::net::IpAddr;

use anyhow::Error as AnyError;
use either::Either;
use ipnet::IpNet;
use itertools::Itertools;

use crate::{input::Input, source::Source};

pub fn process_batch(
    sources: Vec<Source>,
    all: bool,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let input = Input::<IpNet>::Lazy(sources);
    if sort || unique {
        let mut hosts = input
            .into_iter()
            .map(|net| net.map(|net| process_single(net, all)))
            .flatten_ok()
            .collect::<Result<Vec<_>, _>>()?;
        if sort {
            hosts.sort();
        }
        if unique {
            hosts = hosts.into_iter().unique().collect();
        }
        println!("{}", hosts.iter().join("\n"));
    } else {
        for net in input {
            for host in process_single(net?, all) {
                println!("{host}");
            }
        }
    }

    Ok(())
}

fn process_single(net: IpNet, all: bool) -> impl Iterator<Item = IpAddr> {
    if all {
        Either::Left(
            net.subnets(net.max_prefix_len())
                .expect("unexpected invalid subnets")
                .map(|net| net.addr()),
        )
    } else {
        Either::Right(net.hosts())
    }
}
