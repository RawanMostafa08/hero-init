use crate::config::*;
use crate::paths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

// Temporary structs only for serializing correct Netplan format
#[derive(Serialize)]
struct NetplanWrapper {
    network: NetplanInner,
}

#[derive(Serialize)]
struct NetplanInner {
    version: u8,
    renderer: &'static str,
    ethernets: HashMap<String, NetplanEthernet>,
}

#[derive(Serialize)]
struct NetplanEthernet {
    #[serde(skip_serializing_if = "Option::is_none")]
    match_: Option<Match>,
    dhcp4: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    addresses: Option<Vec<String>>,
    #[serde(rename = "gateway4", skip_serializing_if = "Option::is_none")]
    gateway4: Option<String>,
    #[serde(rename = "gateway6", skip_serializing_if = "Option::is_none")]
    gateway6: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    routes: Option<Vec<Route>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nameservers: Option<Vec<NameServer>>,
}

#[derive(Serialize)]
struct Match {
    macaddress: String,
}

// Define the network configuration types
#[derive(Debug, Clone, Copy, Deserialize, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkConfigType {
    #[default]
    Netplan,
    Ifupdown,
}

// Write the network configuration to the appropriate file based on provider
pub fn write_network_config(config_yaml: &str, provider: NetworkConfigType) -> io::Result<()> {
    let (path, content) = match provider {
        NetworkConfigType::Netplan => (paths::NET_PLAN_CONFIG_PATH, config_yaml),
        NetworkConfigType::Ifupdown => (paths::IF_UP_DOWN_CONFIG_PATH, config_yaml),
    };

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

// Apply the network configuration using the appropriate command
pub fn apply_network_config(provider: NetworkConfigType) -> io::Result<()> {
    let status = match provider {
        NetworkConfigType::Netplan => Command::new("netplan").arg("apply").status()?,

        NetworkConfigType::Ifupdown => Command::new("systemctl")
            .args(["restart", "networking"])
            .status()?,
    };

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("failed to apply network configuration"))
    }
}

// Main function to apply network configuration
pub fn apply(network: &Network) -> Result<()> {
    let mut ethernets = HashMap::new();

    for iface in &network.interfaces {
        let netplan_iface = NetplanEthernet {
            match_: Some(Match {
                macaddress: iface.mac.clone(),
            }),
            dhcp4: iface.dhcp4,
            addresses: iface.addresses.clone(),
            gateway4: iface.gateway4.clone(),
            gateway6: iface.gateway6.clone(),
            routes: iface.routes.clone(),
            nameservers: iface.nameservers.clone(),
        };

        ethernets.insert(iface.name.clone(), netplan_iface);
    }

    let wrapper = NetplanWrapper {
        network: NetplanInner {
            version: 2,
            renderer: "networkd",
            ethernets,
        },
    };

    // serialize network config to YAML
    let config_yaml = serde_yaml::to_string(&wrapper)?;
    log::info!("Generated netplan config:\n{}", config_yaml);

    write_network_config(&config_yaml, network.provider)?;

    apply_network_config(network.provider)?;
    Ok(())
}
