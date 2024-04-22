use std::sync::OnceLock;

use anyhow::{bail, Error as AnyError};
use evalexpr::{ContextWithMutableVariables, HashMapContext, Value};
use ipnet::IpNet;
use regex::Regex;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::{addr_or_net::AddrOrNet, configuration::Configuration, input::Input, source::Source};

pub fn process_batch(
    sources: Vec<Source>,
    query: String,
    mut configuration: Option<Configuration>,
    sort: bool,
    unique: bool,
) -> Result<(), AnyError> {
    let mut input = Input::<AddrOrNet>::Lazy(sources);
    if unique {
        input.unique()?;
    }
    if sort {
        input.sort()?;
    }

    let requested_placeholders = Placeholder::requested(&query).collect::<Vec<_>>();
    for value in input {
        let value = value?;

        let mut context = HashMapContext::new();
        for placeholder in &requested_placeholders {
            context.set_value(
                <&str>::from(placeholder).to_owned(),
                placeholder.resolve(value, configuration.as_mut())?,
            )?;
        }

        if evalexpr::eval_boolean_with_context_mut(&query, &mut context)? {
            println!("{}", value);
        }
    }

    Ok(())
}

#[derive(IntoStaticStr, EnumIter, Copy, Clone, Debug)]
#[strum(serialize_all = "snake_case")]
enum Placeholder {
    IpVersion,
    Type,
    Prefix,
    Group,
    Groups,
    Hosts,
}

impl Placeholder {
    fn requested(query: &str) -> impl Iterator<Item = Placeholder> + '_ {
        static CELL: OnceLock<Vec<(Placeholder, Regex)>> = OnceLock::new();
        CELL.get_or_init(|| {
            Placeholder::iter()
                .map(|p| {
                    (
                        p,
                        Regex::new(&format!(r#"\b{}\b"#, <&str>::from(p))).expect("invalid regex"),
                    )
                })
                .collect()
        })
        .iter()
        .filter_map(|(p, r)| r.is_match(query).then_some(*p))
    }

    fn resolve(
        self,
        input: AddrOrNet,
        configuration: Option<&mut Configuration>,
    ) -> Result<Value, AnyError> {
        Ok(match self {
            Placeholder::IpVersion => Value::Int(match IpNet::from(input) {
                IpNet::V4(_) => 4,
                IpNet::V6(_) => 6,
            }),
            Placeholder::Type => Value::String(
                match input {
                    AddrOrNet::IpAddr(_) => "addr",
                    AddrOrNet::IpNet(_) => "net",
                }
                .to_owned(),
            ),
            Placeholder::Prefix => Value::Int(IpNet::from(input).prefix_len() as i64),
            Placeholder::Group => {
                match matching_groups(input, configuration)?.next().transpose()? {
                    Some(group) => Value::String(group.to_owned()),
                    None => Value::Empty,
                }
            }
            Placeholder::Groups => Value::Tuple(
                matching_groups(input, configuration)?
                    .map(|group| group.map(|group| Value::String(group.to_owned())))
                    .collect::<Result<_, _>>()?,
            ),
            Placeholder::Hosts => {
                let net = IpNet::from(input);
                Value::Int(2u128.pow((net.max_prefix_len() - net.prefix_len()) as u32) as i64)
            }
        })
    }
}

fn matching_groups(
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
