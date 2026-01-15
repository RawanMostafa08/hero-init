use crate::config::User;
use crate::paths;
use anyhow::Result;
use log::info;
use nix::unistd::{Gid, Uid, chown};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

// Create user if not exists
pub fn add_system_user(username: &str, shell: &str, groups: &[&str]) -> Result<()> {
    // Check if user already exists
    if Command::new("id").arg(username).status()?.success() {
        info!("User {} already exists, skipping creation", username);
        return Ok(());
    }

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
    if keys.is_empty() {
        info!("No SSH keys for {}, skipping", username);
        return Ok(());
    }
    let home = resolve_home_dir(username, paths::PASSWD_FILE_PATH)?;
    let ssh_dir = home.join(".ssh");
    fs::create_dir_all(&ssh_dir)?;

    let auth_keys = ssh_dir.join("authorized_keys");
    let content = keys.join("\n") + "\n";
    fs::write(&auth_keys, content)?;
    Ok(())
}

fn resolve_home_dir(username: &str, path: &str) -> Result<PathBuf> {
    let passwd = fs::read_to_string(path)?;

    // ‘/etc/passwd’, the passwd file consist of user information includes seven columns separated by colons.
    // Username:password:user-id:group-id:user-info:home-directory:login-shell

    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 6 && parts[0] == username {
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
    add_sudo_rule_with_path(username, paths::SUDOERS_PATH)
}

fn add_sudo_rule_with_path(username: &str, sudoers_path: &str) -> Result<()> {
    let path = format!("{}/{}", sudoers_path, username);
    let rule = format!("{} ALL=(ALL) NOPASSWD:ALL\n", username);

    fs::create_dir_all(sudoers_path)?;
    fs::write(&path, rule)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o440))?;

    info!("Added sudoers rule for {}", username);
    Ok(())
}

// Get user IDs from /etc/passwd
fn get_user_ids(username: &str, path: &str) -> Result<(u32, u32)> {
    let passwd = fs::read_to_string(path)?;

    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 && parts[0] == username {
            let uid = parts[2].parse::<u32>()?;
            let gid = parts[3].parse::<u32>()?;
            return Ok((uid, gid));
        }
    }

    anyhow::bail!("User {} not found in /etc/passwd", username)
}

// Apply user configurations
pub fn apply(users: &[User]) -> Result<()> {
    if users.is_empty() {
        info!("No users to configure");
        return Ok(());
    }

    for user in users {
        let groups_ref: Vec<&str> = user.groups.iter().map(|s| s.as_str()).collect();
        add_system_user(&user.name, paths::BASH_PATH, &groups_ref)?;

        inject_ssh_keys(&user.name, &user.ssh_authorized_keys)?;

        // Set proper permissions for .ssh
        let home_dir = resolve_home_dir(&user.name, paths::PASSWD_FILE_PATH)?;
        let (uid, gid) = get_user_ids(&user.name, paths::PASSWD_FILE_PATH)?;

        let ssh_dir = home_dir.join(".ssh");
        let auth_keys = ssh_dir.join("authorized_keys");

        // Set .ssh directory: owned by user, mode 700
        if !user.ssh_authorized_keys.is_empty() {
            set_permissions(&ssh_dir, 0o700, uid, gid)?;

            // Set authorized_keys: owned by user, mode 600
            if auth_keys.exists() {
                set_permissions(&auth_keys, 0o600, uid, gid)?;
            }
        }

        set_permissions(&home_dir, 0o755, uid, gid)?;

        if user.sudo {
            add_sudo_rule(&user.name)?;
        }
    }
    Ok(())
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_home_dir() {
        let temp_dir = TempDir::new().unwrap();
        let passwd_path = temp_dir.path().join("passwd");
        let sample_passwd = "testuser:x:1000:1000:Test User:/home/testuser:/bin/bash\n";
        fs::write(&passwd_path, sample_passwd).unwrap();

        let home = resolve_home_dir("testuser", passwd_path.to_str().unwrap()).unwrap();
        assert_eq!(home, PathBuf::from("/home/testuser"));
    }

    #[test]
    fn test_get_user_ids() {
        let temp_dir = TempDir::new().unwrap();
        let passwd_path = temp_dir.path().join("passwd");
        let sample_passwd = "testuser:x:1000:1000:Test User:/home/testuser:/bin/bash\n";
        fs::write(&passwd_path, sample_passwd).unwrap();

        let (uid, gid) = get_user_ids("testuser", passwd_path.to_str().unwrap()).unwrap();
        assert_eq!(uid, 1000);
        assert_eq!(gid, 1000);
    }

    #[test]
    fn test_set_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        let _ = set_permissions(&test_file, 0o644, 1000, 1000); // chown may fail in non-root CI

        let metadata = fs::metadata(&test_file).unwrap();
        let mode = metadata.permissions().mode();
        // mask with 0o7777 to get permission bits
        assert_eq!(mode & 0o7777, 0o644);
    }

    #[test]
    fn test_add_sudo_rule() {
        let temp_dir = TempDir::new().unwrap();
        let sudoers_dir = temp_dir.path().join("sudoers");
        fs::create_dir_all(&sudoers_dir).unwrap();
        add_sudo_rule_with_path("testuser", sudoers_dir.to_str().unwrap()).unwrap();
        let rule_path = Path::new(sudoers_dir.to_str().unwrap()).join("testuser");
        let rule_content = fs::read_to_string(&rule_path).unwrap();
        assert_eq!(rule_content, "testuser ALL=(ALL) NOPASSWD:ALL\n");
    }

    #[test]
    fn test_apply_no_users() {
        let users = vec![];
        let result = apply(&users);
        assert!(result.is_ok());
    }
}
