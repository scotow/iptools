use anyhow::{bail, Error as AnyError};

use crate::{addr_or_net::AddrOrNet, configuration::Configuration};

pub fn matching_groups(
    input: AddrOrNet,
    configuration: Option<&mut Configuration>,
) -> Result<impl Iterator<Item = Result<&str, AnyError>>, AnyError> {
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
