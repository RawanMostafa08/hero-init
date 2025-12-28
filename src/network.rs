use crate::config::Network;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, Deserialize, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkConfigType {
    #[default]
    Netplan,
    Ifupdown,
}

pub fn write_network_config(config_yaml: &str, provider: NetworkConfigType) -> io::Result<()> {
    let (path, content) = match provider {
        NetworkConfigType::Netplan => ("/etc/netplan/01-hero-init.yaml", config_yaml),
        NetworkConfigType::Ifupdown => ("/etc/network/interfaces", config_yaml),
    };

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

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
        Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to apply network configuration",
        ))
    }
}

pub fn apply(network: &Network) -> Result<()> {
    // serialize network config to YAML
    let config_yaml = serde_yaml::to_string(&network.interfaces)?;

    write_network_config(&config_yaml, network.provider)?;

    apply_network_config(network.provider)?;
    Ok(())
}
