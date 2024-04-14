use std::{net::IpAddr, str::FromStr};

use anyhow::Error as AnyError;
use ipnet::IpNet;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct AutoNet(pub IpNet);

impl FromStr for AutoNet {
    type Err = AnyError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains('/') {
            Ok(Self(input.parse()?))
        } else {
            Ok(Self(IpNet::from(input.parse::<IpAddr>()?)))
        }
    }
}
