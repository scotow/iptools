use ipnet::IpNet;

use crate::{auto_net::AutoNet, input::Input, source::Source};

pub fn process(sources: Vec<Source>, sort: bool, unique: bool) -> Result<(), anyhow::Error> {
    let mut input = Input::<AutoNet>::Lazy(sources);
    if unique {
        input.unique()?;
    }
    if sort {
        input.sort()?;
    }

    for net in merge(
        &input
            .into_iter()
            .map(|entry| entry.map(|auto_net| auto_net.0.trunc()))
            .collect::<Result<Vec<_>, _>>()?,
    ) {
        println!("{net}");
    }

    Ok(())
}

fn merge(nets: &[IpNet]) -> Vec<IpNet> {
    if nets.is_empty() {
        return Vec::new();
    }

    let mut nets = nets.to_owned();
    loop {
        let mut merged = Vec::with_capacity(nets.len());
        let mut current = nets[0];
        for &net in &nets[1..] {
            if current.contains(&net) {
                continue;
            }
            if net.contains(&current) {
                current = net;
                continue;
            }
            if current.supernet().expect("unexpected top net")
                == net.supernet().expect("unexpected top net")
            {
                current = current.supernet().expect("unexpected top net");
                continue;
            }

            merged.push(current);
            current = net;
        }
        merged.push(current);

        if merged.len() == nets.len() {
            return merged;
        }
        nets = merged;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn contains() {
        assert_eq!(
            super::merge(&[
                "10.0.0.0/31".parse().unwrap(),
                "10.0.0.1/32".parse().unwrap(),
            ]),
            ["10.0.0.0/31".parse().unwrap(),]
        );
        assert_eq!(
            super::merge(&[
                "10.0.0.0/24".parse().unwrap(),
                "10.0.0.1/32".parse().unwrap(),
            ]),
            ["10.0.0.0/24".parse().unwrap()]
        );
        // Other way.
        assert_eq!(
            super::merge(&[
                "10.0.0.1/32".parse().unwrap(),
                "10.0.0.0/24".parse().unwrap(),
            ]),
            ["10.0.0.0/24".parse().unwrap()]
        );
    }

    #[test]
    fn supernet() {
        assert_eq!(
            super::merge(&[
                "10.0.0.0/32".parse().unwrap(),
                "10.0.0.1/32".parse().unwrap(),
            ]),
            ["10.0.0.0/31".parse().unwrap()]
        );
        // Can't be merged.
        assert_eq!(
            super::merge(&[
                "10.0.0.1/32".parse().unwrap(),
                "10.0.0.2/32".parse().unwrap(),
            ]),
            [
                "10.0.0.1/32".parse().unwrap(),
                "10.0.0.2/32".parse().unwrap(),
            ]
        );
    }

    #[test]
    fn list() {
        // 4x/32 into 2x/31 into 1x/30.
        assert_eq!(
            super::merge(&[
                "10.0.0.0/32".parse().unwrap(),
                "10.0.0.1/32".parse().unwrap(),
                "10.0.0.2/32".parse().unwrap(),
                "10.0.0.3/32".parse().unwrap(),
            ]),
            ["10.0.0.0/30".parse().unwrap()]
        );

        // Only consecutive nets are merged.
        assert_eq!(
            super::merge(&[
                "10.0.0.0/32".parse().unwrap(),
                "10.0.1.0/32".parse().unwrap(),
                "10.0.0.1/32".parse().unwrap(),
            ]),
            [
                "10.0.0.0/32".parse().unwrap(),
                "10.0.1.0/32".parse().unwrap(),
                "10.0.0.1/32".parse().unwrap(),
            ]
        );

        // Complex list.
        assert_eq!(
            super::merge(&[
                "10.0.0.0/25".parse().unwrap(),
                "10.0.0.128/25".parse().unwrap(),
                "10.0.1.0/24".parse().unwrap(),
                "10.0.2.0/24".parse().unwrap(),
                "10.0.3.0/24".parse().unwrap(),
                "10.0.4.0/23".parse().unwrap(),
                "10.0.6.0/24".parse().unwrap(),
                "10.0.7.0/24".parse().unwrap(),
                "10.0.8.0/22".parse().unwrap(),
                "10.0.12.0/24".parse().unwrap(),
                "10.0.14.0/23".parse().unwrap(),
                "172.16.0.0/24".parse().unwrap(),
                "172.16.1.0/24".parse().unwrap(),
                "172.16.2.0/24".parse().unwrap(),
                "172.16.3.0/24".parse().unwrap(),
                "172.16.5.0/24".parse().unwrap(),
                "192.168.0.0/26".parse().unwrap(),
                "192.168.0.64/26".parse().unwrap(),
                "192.168.0.128/26".parse().unwrap(),
                "192.168.0.192/26".parse().unwrap(),
                "192.168.1.0/25".parse().unwrap(),
                "192.168.1.128/25".parse().unwrap(),
            ]),
            [
                "10.0.0.0/21".parse().unwrap(),
                "10.0.8.0/22".parse().unwrap(),
                "10.0.12.0/24".parse().unwrap(),
                "10.0.14.0/23".parse().unwrap(),
                "172.16.0.0/22".parse().unwrap(),
                "172.16.5.0/24".parse().unwrap(),
                "192.168.0.0/23".parse().unwrap(),
            ]
        );
    }
}
