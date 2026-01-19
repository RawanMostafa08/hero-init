use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub metadata: Metadata,
    #[serde(default)]
    pub network: Option<Network>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub runcmd: Vec<RunCommand>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Network {
    pub interfaces: Vec<Ethernet>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub instance_id: String,
    pub hostname: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub name: String,
    #[serde(default)]
    pub ssh_authorized_keys: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub sudo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route {
    pub to: String,
    pub via: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NameServers {
    pub search: Vec<String>,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ethernet {
    pub name: String,
    pub mac: String,
    pub dhcp4: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<Route>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<NameServers>,
}
