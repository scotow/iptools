use std::{net::IpAddr, str::FromStr};

use anyhow::Error as AnyError;
use ipnet::IpNet;
use serde::{de, Deserialize, Deserializer};

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

impl<'de> Deserialize<'de> for AutoNet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FromStr::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}
