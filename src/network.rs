use crate::config::*;
use crate::paths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
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
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
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
    nameservers: Option<NameServers>,
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

fn write_network_config_with_path(config_yaml: &str, path: &str) -> io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, config_yaml)?;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;

    Ok(())
}

// Write the network configuration to the appropriate file based on provider
pub fn write_network_config(config_yaml: &str, provider: NetworkConfigType) -> io::Result<()> {
    let path = match provider {
        NetworkConfigType::Netplan => paths::NET_PLAN_CONFIG_PATH,
        NetworkConfigType::Ifupdown => paths::IF_UP_DOWN_CONFIG_PATH,
    };

    write_network_config_with_path(config_yaml, path)
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
        Err(io::Error::other("failed to apply network configuration."))
    }
}

fn wrap_network(interfaces: &Vec<Ethernet>) -> Result<NetplanWrapper> {
    let mut ethernets = HashMap::new();

    for iface in interfaces {
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

    Ok(wrapper)
}

// serialize network config to YAML
fn net_to_yaml(wrapper: &NetplanWrapper) -> Result<String> {
    let config_yaml = serde_yaml::to_string(wrapper)?;
    Ok(config_yaml)
}

// Main function to apply network configuration
pub fn apply(network: &Network) -> Result<()> {
    let wrapper = wrap_network(&network.interfaces);

    let config_yaml = net_to_yaml(&wrapper.unwrap())?;

    write_network_config(&config_yaml, network.provider)?;

    apply_network_config(network.provider)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_apply_generates_valid_netplan_config() {
        let network = Network {
            provider: NetworkConfigType::Netplan,
            interfaces: vec![
                Ethernet {
                    name: "eth0".to_string(),
                    mac: "52:54:00:12:34:56".to_string(),
                    dhcp4: true,
                    addresses: None,
                    gateway4: None,
                    gateway6: None,
                    routes: None,
                    nameservers: None,
                },
                Ethernet {
                    name: "ens3".to_string(),
                    mac: "00:16:3e:aa:bb:cc".to_string(),
                    dhcp4: false,
                    addresses: Some(vec!["10.0.0.10/24".to_string()]),
                    gateway4: Some("10.0.0.1".to_string()),
                    gateway6: None,
                    routes: None,
                    nameservers: Some(NameServers {
                        search: vec!["example.com".to_string()],
                        addresses: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()],
                    }),
                },
            ],
        };

        let wrapper = wrap_network(&network.interfaces);

        let config_yaml = net_to_yaml(&wrapper.unwrap());

        let temp_dir = TempDir::new().unwrap();
        let test_output_path = temp_dir.path().join("test-netplan-output.yaml");
        let test_output_path_str = test_output_path
            .to_str()
            .expect("Failed to convert temp path to string");

        write_network_config_with_path(&config_yaml.unwrap(), test_output_path_str)
            .expect("Failed to write test output file");

        let content = fs::read_to_string(test_output_path).expect("Failed to read generated file");

        // Must start with the top-level 'network:' key
        assert!(
            content.trim_start().starts_with("network:"),
            "Generated YAML does not start with 'network:'\n---\n{}\n---",
            content
        );

        // Must contain version 2
        assert!(
            content.contains("version: 2"),
            "Missing 'version: 2'\n---\n{}\n---",
            content
        );

        // Must contain the interface names as keys under ethernets
        assert!(
            content.contains("eth0:"),
            "Missing eth0 interface\n---\n{}\n---",
            content
        );
        assert!(
            content.contains("ens3:"),
            "Missing ens3 interface\n---\n{}\n---",
            content
        );

        // Must contain match with macaddress
        assert!(
            content.contains("macaddress: 00:16:3e:aa:bb:cc"),
            "Missing correct macaddress for eth0\n---\n{}\n---",
            content
        );

        // Optional fields that are None should be completely omitted (not "null")
        assert!(
            !content.contains("addresses: null"),
            "Found 'null' for addresses – optional fields should be skipped\n---\n{}\n---",
            content
        );
    }
}
