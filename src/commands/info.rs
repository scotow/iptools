use std::net::IpAddr;
use anyhow::Error as AnyError;

use indoc::formatdoc;
use ipnet::IpNet;
use itertools::Itertools;
use crate::auto_net::AutoNet;
use crate::input::Input;
use crate::source::Source;

pub fn process_batch(
    sources: Vec<Source>,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let mut input = Input::<AutoNet>::Lazy(sources);
    if sort {
        input = input.sort()?;
    }
    if unique {
        input = input.unique()?;
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
    let mut output = formatdoc! {"
        address: {}
        network: {}
        hosts range: {} - {}",
        addr.addr(),
        addr.network(),
        addr.hosts().next().unwrap(),
        addr.hosts().rev().next().unwrap(),
    };
    if addr.addr().is_ipv4() {
        output.push_str(&format!("\nbroadcast: {}", addr.broadcast()));
    }
    output.push_str(&format!(
        "\nhosts: {}",
        2u128.pow((addr.max_prefix_len() - addr.prefix_len()) as u32)
    ));
    if addr.addr().is_ipv4() {
        output.push_str(&format!("\nusable hosts: {}", addr.hosts().count()));
    }
    output.push('\n');
    output.push_str(&formatdoc! {"
        net mask: {}
        host mask: {}
        CIDR: /{}
        full: {}
        binary address: {}
        binary net mask: {}",
        addr.netmask(),
        addr.hostmask(),
        addr.prefix_len(),
        addr,
        to_binary(addr.addr()),
        to_binary(addr.netmask()),
    });
    match addr.addr() {
        IpAddr::V4(addr) => {
            output.push_str(&format!("\nipv6 mapping: {}", addr.to_ipv6_compatible()));
        }
        IpAddr::V6(_) => {}
    }
    output
}

fn to_binary(addr: IpAddr) -> String {
    let as_str = addr.to_string();
    match addr {
        IpAddr::V4(_addr) => as_str
            .split('.')
            .map(|p| format!("{:08b}", p.parse::<u8>().unwrap()))
            .join("."),
        IpAddr::V6(_addr) => as_str
            .split(':')
            .map(|p| {
                if p.is_empty() {
                    p.to_owned()
                } else {
                    format!("{:016b}", u16::from_str_radix(p, 16).unwrap())
                }
            })
            .join(":"),
    }
}
