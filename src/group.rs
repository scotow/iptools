use anyhow::bail;

use crate::{addr_or_net::AddrOrNet, config::Config};

pub fn matching_groups(
    input: AddrOrNet,
    configuration: Option<&mut Config>,
) -> Result<impl Iterator<Item = Result<&str, anyhow::Error>>, anyhow::Error> {
    let groups = match configuration {
        Some(configuration) => match &mut configuration.groups {
            Some(groups) => groups,
            None => bail!("no groups defined in configuration"),
        },
        None => bail!("configuration required to filter based on groups"),
    };

    Ok(groups.iter_mut().flat_map(move |group| {
        group
            .source
            .load()
            .map(|nets| {
                nets.iter()
                    .any(|net| match input {
                        AddrOrNet::IpAddr(addr) => net.0.contains(&addr),
                        AddrOrNet::IpNet(sub_net) => net.0.contains(&sub_net),
                    })
                    .then_some(group.name.as_str())
            })
            .transpose()
    }))
}
