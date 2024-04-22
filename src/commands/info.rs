use std::{fmt, fmt::Write, net::IpAddr};

use anyhow::Error as AnyError;
use ipnet::IpNet;
use itertools::Itertools;

use crate::{auto_net::AutoNet, input::Input, source::Source};

const DEFAULT_IPV4_FIELDS: &[Field] = &[
    Field::Address,
    Field::NetworkAddress,
    Field::HostsRange,
    Field::BroadcastAddress,
    Field::HostsCount,
    Field::UsableHostsCount,
    Field::NetworkMaskAddress,
    Field::HostMaskAddress,
    Field::Cidr,
    Field::FullAddress,
    Field::BinaryAddress,
    Field::BinaryNetworkMaskAddress,
    Field::Ipv6Mapping,
];
const DEFAULT_IPV6_FIELDS: &[Field] = &[
    Field::Address,
    Field::NetworkAddress,
    Field::HostsRange,
    Field::HostsCount,
    Field::NetworkMaskAddress,
    Field::HostMaskAddress,
    Field::Cidr,
    Field::FullAddress,
    Field::BinaryAddress,
    Field::BinaryNetworkMaskAddress,
];

pub fn process_batch(sources: Vec<Source>, sort: bool, unique: bool) -> Result<(), AnyError> {
    let mut input = Input::<AutoNet>::Lazy(sources);
    if sort {
        input.sort()?;
    }
    if unique {
        input.unique()?;
    }

    #[allow(unstable_name_collisions)]
    for value in input
        .into_iter()
        .map(|addr| addr.map(|addr| process(addr.0)))
        .intersperse_with(|| Ok(String::new()))
    {
        println!("{}", value?);
    }

    Ok(())
}

fn process(addr: IpNet) -> String {
    let fields = match addr {
        IpNet::V4(_) => DEFAULT_IPV4_FIELDS,
        IpNet::V6(_) => DEFAULT_IPV6_FIELDS,
    };
    let mut buffer = String::with_capacity(1024);
    let tail = match fields.split_last() {
        Some((tail, head)) => {
            for field in head {
                field.write(addr, &mut buffer).unwrap();
                buffer.push('\n');
            }
            tail
        }
        None => &fields[0],
    };
    tail.write(addr, &mut buffer).unwrap();
    buffer
}

#[derive(Copy, Clone)]
enum Field {
    Address,
    NetworkAddress,
    HostsRange,
    BroadcastAddress,
    HostsCount,
    UsableHostsCount,
    NetworkMaskAddress,
    HostMaskAddress,
    Cidr,
    FullAddress,
    BinaryAddress,
    BinaryNetworkMaskAddress,
    Ipv6Mapping,
}

impl Field {
    fn write(self, addr: IpNet, s: &mut String) -> fmt::Result {
        match self {
            Field::Address => write!(s, "address: {}", addr.addr()),
            Field::NetworkAddress => write!(s, "network: {}", addr.network()),
            Field::HostsRange => write!(
                s,
                "hosts range: {} - {}",
                addr.hosts().next().unwrap(),
                addr.hosts().next_back().unwrap()
            ),
            Field::BroadcastAddress => write!(s, "broadcast: {}", addr.broadcast()),
            Field::HostsCount => write!(
                s,
                "hosts: {}",
                2u128.pow((addr.max_prefix_len() - addr.prefix_len()) as u32)
            ),
            Field::UsableHostsCount => write!(s, "usable hosts: {}", addr.hosts().count()),
            Field::NetworkMaskAddress => write!(s, "net mask: {}", addr.netmask()),
            Field::HostMaskAddress => write!(s, "host mask: {}", addr.hostmask()),
            Field::Cidr => write!(s, "CIDR: {}", addr.prefix_len()),
            Field::FullAddress => write!(s, "full: {}", addr),
            Field::BinaryAddress => write!(s, "binary address: {}", to_binary(addr.addr())),
            Field::BinaryNetworkMaskAddress => {
                write!(s, "binary net mask: {}", to_binary(addr.netmask()))
            }
            Field::Ipv6Mapping => match addr {
                IpNet::V4(addr) => write!(s, "IPv6 mapping: {}", addr.addr().to_ipv6_compatible()),
                IpNet::V6(_) => Field::Address.write(addr, s),
            },
        }
    }
}

fn to_binary(addr: IpAddr) -> String {
    let as_str = addr.to_string();
    match addr {
        IpAddr::V4(_) => as_str
            .split('.')
            .map(|p| format!("{:08b}", p.parse::<u8>().unwrap()))
            .join("."),
        IpAddr::V6(_) => as_str
            .split(':')
            .map(|p| {
                if p.is_empty() {
                    String::new()
                } else {
                    format!("{:016b}", u16::from_str_radix(p, 16).unwrap())
                }
            })
            .join(":"),
    }
}
