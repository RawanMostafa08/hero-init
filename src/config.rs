use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub metadata: Metadata,
    pub network: Vec<Ethernet>,
    pub users: Vec<User>,
    pub mounts: Vec<Mount>,
    pub extension: Extension,
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
}

#[derive(Debug, Deserialize, Default)]
pub struct Extension {
    pub entrypoint: String,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Mount {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Route {
    pub to: String,
    pub via: String,
    pub metric: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NameServer {
    pub search: Vec<String>,
    pub addresses: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
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
