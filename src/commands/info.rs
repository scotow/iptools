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

pub fn process_batch(
    sources: Vec<Source>,
    padding: bool,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let mut input = Input::<AutoNet>::Lazy(sources);
    if unique {
        input.unique()?;
    }
    if sort {
        input.sort()?;
    }

    #[allow(unstable_name_collisions)]
    for value in input
        .into_iter()
        .map(|addr| addr.map(|addr| process(addr.0, padding)))
        .intersperse_with(|| Ok(String::new()))
    {
        println!("{}", value?);
    }

    Ok(())
}

fn process(addr: IpNet, padding: bool) -> String {
    let fields = match addr {
        IpNet::V4(_) => DEFAULT_IPV4_FIELDS,
        IpNet::V6(_) => DEFAULT_IPV6_FIELDS,
    };
    let mut buffer = String::with_capacity(1024);
    let label_max_len = padding.then(|| fields.iter().map(|f| f.label().len()).max().unwrap());
    let tail = match fields.split_last() {
        Some((tail, head)) => {
            for field in head {
                field.write(addr, &mut buffer, label_max_len).unwrap();
                buffer.push('\n');
            }
            tail
        }
        None => &fields[0],
    };
    tail.write(addr, &mut buffer, label_max_len).unwrap();
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
    fn write(self, addr: IpNet, s: &mut String, label_max_len: Option<usize>) -> fmt::Result {
        let label = self.label();
        write!(
            s,
            "{label}:{:1$}",
            "",
            label_max_len.map(|m| m - label.len()).unwrap_or(0) + 1
        )?;
        match self {
            Field::Address => write!(s, "{}", addr.addr()),
            Field::NetworkAddress => write!(s, "{}", addr.network()),
            Field::HostsRange => write!(
                s,
                "{} - {}",
                addr.hosts().next().unwrap(),
                addr.hosts().next_back().unwrap()
            ),
            Field::BroadcastAddress => write!(s, "{}", addr.broadcast()),
            Field::HostsCount => write!(
                s,
                "{}",
                2u128.pow((addr.max_prefix_len() - addr.prefix_len()) as u32)
            ),
            Field::UsableHostsCount => write!(s, "{}", addr.hosts().count()),
            Field::NetworkMaskAddress => write!(s, "{}", addr.netmask()),
            Field::HostMaskAddress => write!(s, "{}", addr.hostmask()),
            Field::Cidr => write!(s, "{}", addr.prefix_len()),
            Field::FullAddress => write!(s, "{}", addr),
            Field::BinaryAddress => write!(s, "{}", to_binary(addr.addr())),
            Field::BinaryNetworkMaskAddress => {
                write!(s, "{}", to_binary(addr.netmask()))
            }
            Field::Ipv6Mapping => match addr {
                IpNet::V4(addr) => write!(s, "{}", addr.addr().to_ipv6_compatible()),
                IpNet::V6(_) => Field::Address.write(addr, s, label_max_len),
            },
        }
    }

    fn label(self) -> &'static str {
        match self {
            Field::Address => "address",
            Field::NetworkAddress => "network",
            Field::HostsRange => "hosts ranges",
            Field::BroadcastAddress => "broadcast",
            Field::HostsCount => "hosts",
            Field::UsableHostsCount => "usable hosts",
            Field::NetworkMaskAddress => "net mask",
            Field::HostMaskAddress => "host mask",
            Field::Cidr => "cidr",
            Field::FullAddress => "full",
            Field::BinaryAddress => "binary address",
            Field::BinaryNetworkMaskAddress => "binary net mask",
            Field::Ipv6Mapping => "IPv6 mapping",
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
