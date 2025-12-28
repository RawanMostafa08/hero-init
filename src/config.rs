use serde::{Deserialize, Serialize};

use crate::network::NetworkConfigType;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub metadata: Metadata,
    pub network: Network,
    pub users: Vec<User>,
    pub mounts: Vec<Mount>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Network {
    pub interfaces: Vec<Ethernet>,
    pub provider: NetworkConfigType,
}

#[derive(Debug, Deserialize, Default)]
pub struct Metadata {
    pub instance_id: String,
    pub hostname: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct User {
    pub name: String,
    pub ssh_authorized_keys: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Mount {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Route {
    pub to: String,
    pub via: String,
    pub metric: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NameServer {
    pub search: Vec<String>,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Ethernet {
    pub name: String,
    pub mac: String,
    pub dhcp4: bool,
    pub addresses: Option<Vec<String>>,
    pub gateway4: Option<String>,
    pub gateway6: Option<String>,
    pub routes: Option<Vec<Route>>,
    pub nameservers: Option<Vec<NameServer>>,
}
