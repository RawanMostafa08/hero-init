use crate::config::User;
use anyhow::Result;
use std::fs;
use std::process::Command;

// Create user if not exists
fn useradd(username: &str) -> Result<()> {
    let status = Command::new("useradd")
        .args(["-m", "-s", "/bin/bash", username])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to create user: {}", username);
    }
    Ok(())
}

// Set up SSH authorized keys
fn write_ssh(user: &User, path: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    let auth_keys = format!("{}/authorized_keys", path);
    fs::write(auth_keys, user.ssh_authorized_keys.join("\n"))?;
    Ok(())
}

// Set ownership of ssh directory to the user
fn chown(user: &str, path: &str) -> Result<()> {
    let status = Command::new("chown")
        .args(["-R", &format!("{}:{}", user, user), path])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to chown {} for user {}", path, user);
    }
    Ok(())
}

pub fn apply(users: &[User]) -> Result<()> {
    for user in users {
        // TODO: only support root users if needed
        useradd(&user.name)?;
        let ssh_dir = format!("/home/{}/.ssh", user.name);

        write_ssh(user, &ssh_dir)?;
        chown(&user.name, &ssh_dir)?;
    }
    Ok(())
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::User;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_apply_users_in_tempdir() {
        let temp_dir = TempDir::new().unwrap();
        let ssh_dir = temp_dir.path().join("root/.ssh");

        let users = vec![User {
            name: "root".to_string(),
            ssh_authorized_keys: vec!["ssh-rsa AAA...".to_string()],
        }];

        for user in &users {
            // Write SSH keys to tempdir instead of /home
            write_ssh(user, ssh_dir.to_str().unwrap()).unwrap();
        }

        // Assert that the authorized_keys file exists and has the right content
        let auth_keys_path = ssh_dir.join("authorized_keys");
        let content = fs::read_to_string(auth_keys_path).unwrap();
        assert!(content.contains("ssh-rsa AAA..."));
    }
}
