use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::network::NetworkConfigType;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub metadata: Metadata,
    pub network: Network,
    pub users: Vec<User>,
    #[serde(default)]
    pub runcmd: Vec<RunCommand>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RunCommand {
    // Simple command string (e.g., "echo hello")
    Simple(String),
    // Structured command with optional environment variables
    Structured {
        cmd: String,
        #[serde(default)]
        env: HashMap<String, String>,
    },
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
    pub sudo: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Route {
    pub to: String,
    pub via: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NameServers {
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
    pub nameservers: Option<NameServers>,
}
