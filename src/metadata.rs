use crate::config::Configuration;
use anyhow::Result;
use std::fs;
use std::process::Command;

pub fn apply(cfg: &Configuration) -> Result<()> {
    // Set hostname in /etc/hostname
    fs::write("/etc/hostname", &cfg.metadata.hostname)?;

    // Apply hostname using hostnamectl
    Command::new("hostnamectl")
        .arg("set-hostname")
        .arg(&cfg.metadata.hostname)
        .status()?;

    // Write persistent instance-id
    fs::create_dir_all("/var/lib/hero-init")?;
    fs::write("/var/lib/hero-init/instance-id", &cfg.metadata.instance_id)?;
    Ok(())
}
