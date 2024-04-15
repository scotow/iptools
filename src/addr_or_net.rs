use std::{
    fmt::{Display, Formatter},
    net::IpAddr,
    str::FromStr,
};

use anyhow::Error as AnyError;
use ipnet::IpNet;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum AddrOrNet {
    IpAddr(IpAddr),
    IpNet(IpNet),
}

impl FromStr for AddrOrNet {
    type Err = AnyError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains('/') {
            Ok(Self::IpNet(input.parse()?))
        } else {
            Ok(Self::IpAddr(input.parse()?))
        }
    }
}

impl Display for AddrOrNet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddrOrNet::IpAddr(addr) => write!(f, "{addr}"),
            AddrOrNet::IpNet(net) => write!(f, "{net}"),
        }
    }
}
