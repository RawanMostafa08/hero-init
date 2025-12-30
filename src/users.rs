use crate::config::User;
use crate::paths;
use anyhow::Result;
use nix::unistd::{Gid, Uid, chown};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

// Create user if not exists
pub fn add_system_user(username: &str, shell: &str, groups: &[&str]) -> Result<()> {
    let mut cmd = Command::new("useradd");
    cmd.args(["-m", "-s", shell]);

    if !groups.is_empty() {
        cmd.args(["-G", &groups.join(",")]);
    }

    let status = cmd.arg(username).status()?;

    if !status.success() {
        anyhow::bail!("Failed to create user: {}", username);
    }
    Ok(())
}

// Set up SSH authorized keys
pub fn inject_ssh_keys(username: &str, keys: &[String]) -> Result<()> {
    let home = resolve_home_dir(username)?;
    let ssh_dir = home.join(".ssh");
    fs::create_dir_all(&ssh_dir)?;

    let auth_keys = ssh_dir.join("authorized_keys");
    fs::write(&auth_keys, keys.join("\n"))?;
    Ok(())
}

fn resolve_home_dir(username: &str) -> Result<PathBuf> {
    let passwd = fs::read_to_string(paths::PASSWD_FILE_PATH)?;

    // ‘/etc/passwd’, the passwd file consist of user information includes seven columns separated by colons.
    // Username:password:user-id:group-id:user-info:home-directory:login-shell

    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() > 5 && parts[0] == username {
            return Ok(PathBuf::from(parts[5])); // home-directory
        }
    }

    anyhow::bail!("User {} not found in passwd file", username);
}

// Set file permissions and ownership
pub fn set_permissions(path: &Path, mode: u32, uid: u32, gid: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;
    Ok(())
}

// Add sudo rule for user
pub fn add_sudo_rule(username: &str) -> Result<()> {
    let path = format!("{}/{}", paths::SUDOERS_PATH, username);
    let rule = format!("{} ALL=(ALL) NOPASSWD:ALL\n", username);

    fs::write(&path, rule)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o440))?;

    Ok(())
}

pub fn apply(users: &[User]) -> Result<()> {
    for user in users {
        // TODO: only support root users if needed
        let groups_ref: Vec<&str> = user.groups.iter().map(|s| s.as_str()).collect();
        add_system_user(&user.name, paths::BASH_PATH, &groups_ref)?;

        inject_ssh_keys(&user.name, &user.ssh_authorized_keys)?;

        // Set proper permissions for .ssh
        let home_dir = resolve_home_dir(&user.name)?;
        set_permissions(&home_dir.join(".ssh"), 0o700, 0, 0)?;
        set_permissions(&home_dir.join(paths::AUTH_KEYS_PATH), 0o600, 0, 0)?;

        if user.sudo {
            add_sudo_rule(&user.name)?;
        }
    }
    Ok(())
}
