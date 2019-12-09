#[macro_use]
extern crate serde;
extern crate serde_yaml;
extern crate toml;

use std::collections::{HashMap, BTreeMap};
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use std::error::Error;
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Community {
    pub asn: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tech_c: Vec<String>,
    pub networks: Networks,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub bgp: HashMap<String, Peer>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nameservers: Vec<IpAddr>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub delegate: HashMap<u32, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Networks {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ipv4: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ipv6: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Peer {
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv4: Option<Ipv4Addr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv6: Option<Ipv6Addr>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let communities: Result<BTreeMap<String, Community>, Box<dyn Error>> = fs::read_dir("./icvpn-meta")?
        .filter_map(|res| res.ok())
        // get path only
        .map(|entry| entry.path())
        // only file names without extension (all others are no network description)
        .filter(|path| !path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .contains("."))
        // open file and parse content
        .map(|path| -> Result<(String, Community), Box<dyn Error>> {
            let name = String::from(path.file_name().and_then(|name| name.to_str()).unwrap());
            let community = serde_yaml::from_reader(
                fs::File::open(path)?
            )?;
            Ok((name, community))
        })
        .collect();
    let communities = communities?;

    let networks: BTreeMap<_, _> = communities.iter()
        .flat_map(|(name, data)|
            data.networks.ipv4.iter().chain(data.networks.ipv6.iter()).map(move |network| (network, name))
        )
        .collect();

    println!("{}", toml::to_string(&networks)?);

    Ok(())
}
