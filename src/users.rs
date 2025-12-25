use crate::config::Configuration;
use anyhow::Result;
use std::fs;
use std::process::Command;

pub fn apply(cfg: &Configuration) -> Result<()> {
    for user in &cfg.users {
        // Create user if not exists
        Command::new("useradd")
            .args(["-m", "-s", "/bin/bash", &user.name])
            .status()
            .ok();

        // Set up SSH authorized keys
        let ssh_dir = format!("/home/{}/.ssh", user.name);
        fs::create_dir_all(&ssh_dir)?;

        let auth_keys = format!("{}/authorized_keys", ssh_dir);
        fs::write(auth_keys, user.ssh_authorized_keys.join("\n"))?;

        // Set ownership of ssh directory to the user
        Command::new("chown")
            .args(["-R", &format!("{}:{}", user.name, user.name), &ssh_dir])
            .status()?;
    }
    Ok(())
}
