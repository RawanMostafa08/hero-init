use hero_init::*;

use anyhow::Result;
use config::Configuration;
use std::fs;

fn load_config() -> Result<Configuration> {
    let data = fs::read_to_string("/config/hero-init.yaml")?;
    Ok(serde_yaml::from_str(&data)?)
}

fn is_first_boot(cfg: &Configuration) -> Result<bool> {
    let path = "/var/lib/hero-init/instance-id";
    match fs::read_to_string(path) {
        Ok(stored) => Ok(stored.trim() != cfg.metadata.instance_id),
        Err(_) => Ok(true), // file doesn't exist, assume first boot
    }
}

fn main() -> Result<()> {
    systemd_journal_logger::init().unwrap();
    log::info!("hero-init starting");

    let cfg = load_config()?;

    let is_first = is_first_boot(&cfg)?;
    if is_first {
        log::info!("First boot detected, applying configuration");
        metadata::apply(&cfg.metadata)?;
        network::apply(&cfg.network)?;
        users::apply(&cfg.users)?;
    } else {
        log::info!("Subsequent boot, skipping per-instance configuration");
    }

    log::info!("hero-init completed successfully");
    Ok(())
}
