use crate::config::*;
use anyhow::{Result, anyhow};
use log::error;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
struct NetworkInterface {
    name: String,
    mac: String,
}

fn discover_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();

    let sys_net_path = Path::new("/sys/class/net");
    if !sys_net_path.exists() {
        return Err(anyhow!("Network sysfs path not found"));
    }

    for entry in fs::read_dir(sys_net_path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name == "lo" {
            continue;
        }

        let iface_path = entry.path();
        let address_path = iface_path.join("address");

        if !address_path.exists() {
            continue;
        }

        let mac = fs::read_to_string(&address_path)?.trim().to_lowercase();

        interfaces.push(NetworkInterface { name, mac });
    }

    Ok(interfaces)
}

fn bring_interface_up(iface_name: &str) -> Result<()> {
    let output = Command::new("ip")
        .args(["link", "set", "dev", iface_name, "up"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Failed to bring interface {} up: {}", iface_name, stderr);
        return Err(anyhow!(
            "Failed to bring interface {} up: {}",
            iface_name,
            stderr
        ));
    }

    Ok(())
}

fn disable_ipv6_ra(iface_name: &str) -> Result<()> {
    let ra_path = format!("/proc/sys/net/ipv6/conf/{}/accept_ra", iface_name);

    if Path::new(&ra_path).exists() {
        fs::write(&ra_path, "0")?;
    }

    Ok(())
}

fn add_address(iface_name: &str, address: &str) -> Result<()> {
    let output = Command::new("ip")
        .args(["addr", "add", address, "dev", iface_name])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("File exists") {
            return Ok(());
        }
        error!(
            "Failed to add address {} to {}: {}",
            address, iface_name, stderr
        );
        return Err(anyhow!(
            "Failed to add address {} to {}: {}",
            address,
            iface_name,
            stderr
        ));
    }

    Ok(())
}

fn add_route(route: &str, via: &str, metric: Option<u32>) -> Result<()> {
    let mut cmd = Command::new("ip");
    cmd.args(["route", "add", route, "via", via]);

    if let Some(m) = metric {
        cmd.args(["metric", &m.to_string()]);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("File exists") {
            return Ok(());
        }
        error!("Failed to add route {} via {}: {}", route, via, stderr);
        return Err(anyhow!(
            "Failed to add route {} via {}: {}",
            route,
            via,
            stderr
        ));
    }

    Ok(())
}

fn add_default_gateway(gateway: &str) -> Result<()> {
    let output = Command::new("ip")
        .args(["route", "add", "default", "via", gateway])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("File exists") {
            return Ok(());
        }
        error!("Failed to add default gateway {}: {}", gateway, stderr);
        return Err(anyhow!(
            "Failed to add default gateway {}: {}",
            gateway,
            stderr
        ));
    }

    Ok(())
}

fn add_default_gateway_ipv6(gateway: &str) -> Result<()> {
    let output = Command::new("ip")
        .args(["-6", "route", "add", "default", "via", gateway])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("File exists") {
            return Ok(());
        }
        error!("Failed to add default IPv6 gateway {}: {}", gateway, stderr);
        return Err(anyhow!(
            "Failed to add default IPv6 gateway {}: {}",
            gateway,
            stderr
        ));
    }

    Ok(())
}

fn write_resolv_conf(nameservers: &[String]) -> Result<()> {
    let mut content = String::new();

    for ns in nameservers {
        content.push_str(&format!("nameserver {}\n", ns));
    }

    let temp_path = "/etc/resolv.conf.tmp";
    fs::write(temp_path, content)?;
    fs::set_permissions(temp_path, fs::Permissions::from_mode(0o644))?;

    fs::rename(temp_path, "/etc/resolv.conf")?;

    Ok(())
}

fn apply_interface_config(iface: &Ethernet, system_iface: &NetworkInterface) -> Result<()> {
    bring_interface_up(&system_iface.name)?;

    disable_ipv6_ra(&system_iface.name)?;

    if let Some(ref addresses) = iface.addresses {
        for addr in addresses {
            add_address(&system_iface.name, addr)?;
        }
    }

    if let Some(ref routes) = iface.routes {
        for route in routes {
            add_route(&route.to, &route.via, route.metric)?;
        }
    }

    if let Some(ref gw4) = iface.gateway4 {
        add_default_gateway(gw4)?;
    }

    if let Some(ref gw6) = iface.gateway6 {
        add_default_gateway_ipv6(gw6)?;
    }

    Ok(())
}

pub fn apply(network: &Network) -> Result<()> {
    let system_interfaces = discover_interfaces()?;
    let mut interface_map: HashMap<String, NetworkInterface> = HashMap::new();

    for iface in system_interfaces {
        interface_map.insert(iface.mac.clone(), iface);
    }

    bring_interface_up("lo")?;

    let mut all_nameservers: Vec<String> = Vec::new();
    let mut fatal_error = None;

    for config_iface in &network.interfaces {
        let config_mac = config_iface.mac.to_lowercase();

        if let Some(system_iface) = interface_map.get(&config_mac) {
            match apply_interface_config(config_iface, system_iface) {
                Ok(()) => {
                    if let Some(ref ns) = config_iface.nameservers {
                        for addr in &ns.addresses {
                            if !all_nameservers.contains(addr) {
                                all_nameservers.push(addr.clone());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to configure interface {}: {}", system_iface.name, e);
                    fatal_error = Some(e);
                }
            }
        } else {
            error!("No interface found with MAC address {}", config_iface.mac);

            // Log available MAC addresses for debugging
            let available_macs: Vec<String> = interface_map.keys().cloned().collect();
            error!("Available MAC addresses: {:?}", available_macs);
            fatal_error = Some(anyhow!(
                "No interface found with MAC address {}",
                config_iface.mac
            ));
        }
    }

    if !all_nameservers.is_empty()
        && let Err(e) = write_resolv_conf(&all_nameservers)
    {
        error!("Failed to write resolv.conf: {}", e);
        fatal_error = Some(anyhow!("Failed to write resolv.conf: {}", e));
    }

    if let Some(e) = fatal_error {
        return Err(e);
    }

    Ok(())
}
