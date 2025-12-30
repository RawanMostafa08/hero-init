use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Result;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::paths;

// Run a shell command with specified environment variables
pub fn run_cmd(command: &str, env_vars: HashMap<String, String>) -> Result<()> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let status = cmd.status()?;

    if !status.success() {
        Err(io::Error::other("failed to execute command"))
    } else {
        Ok(())
    }
}

// Write content to a file atomically with specified permissions
pub fn write_file_atomic(path: &Path, content: &str, mode: u32) -> Result<()> {
    let temp_path = path.with_extension("tmp");

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    fs::set_permissions(&temp_path, fs::Permissions::from_mode(mode))?;
    fs::rename(temp_path, path)?;

    Ok(())
}

// Add an entry to /etc/fstab and mount it
pub fn mount_fstab_entry(
    label: &str,
    device: &Path,
    mount_point: &Path,
    fs_type: &str,
) -> Result<()> {
    let entry = format!(
        "LABEL={} {} {} {} defaults 0 0\n",
        label,
        device.display(),
        mount_point.display(),
        fs_type
    );

    {
        let mut fstab = OpenOptions::new().append(true).open(paths::FSTAB_PATH)?;

        fstab.write_all(entry.as_bytes())?;
    }

    let status = Command::new("mount").arg("-a").status()?;

    if !status.success() {
        return Err(io::Error::other(
            "mount -a failed after updating filesystem table",
        ));
    }

    Ok(())
}
