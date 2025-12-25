use crate::config::Configuration;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn apply(cfg: &Configuration) -> Result<()> {
    write_metadata_files(cfg, "/etc/hostname", "/var/lib/hero-init/instance-id")?;
    apply_hostname(&cfg.metadata.hostname)?;
    Ok(())
}

pub fn write_metadata_files(
    cfg: &Configuration,
    hostname_path: &str,
    instance_id_path: &str,
) -> Result<()> {
    // Set hostname in /etc/hostname
    if let Some(parent) = Path::new(hostname_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(hostname_path, &cfg.metadata.hostname)?;

    // Write persistent instance-id
    if let Some(parent) = Path::new(instance_id_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(instance_id_path, &cfg.metadata.instance_id)?;
    Ok(())
}

fn apply_hostname(hostname: &str) -> Result<()> {
    // Apply hostname using hostnamectl
    Command::new("hostnamectl")
        .arg("set-hostname")
        .arg(hostname)
        .status()?;
    Ok(())
}
