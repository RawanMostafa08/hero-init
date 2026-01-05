use crate::config::Metadata;
use crate::paths;
use libc::{SYS_sethostname, syscall};
use std::ffi::CString;
use std::fs;
use std::io::Error;
use std::io::Result;
use std::io::Write;

// Set system hostname
pub fn set_hostname(hostname: &str) -> Result<()> {
    // 1- writing to /etc/hostname
    write_hostname_file(hostname, paths::HOSTNAME_PATH)?;
    // 2- using libc syscall
    set_hostname_syscall(hostname)?;
    Ok(())
}

fn write_hostname_file(hostname: &str, path: &str) -> Result<()> {
    fs::write(path, hostname.as_bytes())
}

fn set_hostname_syscall(hostname: &str) -> Result<()> {
    let cstr = CString::new(hostname).unwrap();
    let res = unsafe { syscall(SYS_sethostname, cstr.as_ptr(), hostname.len()) };

    if res == 0 {
        Ok(())
    } else {
        Err(Error::last_os_error())
    }
}

// Write instance ID to persistent storage
pub fn write_instance_id(instance_id: &str) -> Result<()> {
    write_instance_id_with_path(instance_id, paths::INSTANCE_ID_PATH)
}

fn write_instance_id_with_path(instance_id: &str, instance_id_path: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    // Write persistent instance-id
    if let Some(parent) = Path::new(instance_id_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(instance_id_path, instance_id)?;
    Ok(())
}

// Update /etc/hosts file
pub fn update_hosts_file(hostname: &str) -> Result<()> {
    update_hosts_file_with_path(hostname, paths::HOSTS_FILE_PATH)?;
    Ok(())
}

fn update_hosts_file_with_path(hostname: &str, path: &str) -> Result<()> {
    let mut hosts_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    writeln!(hosts_file, "127.0.1.1 {}", hostname)?;
    Ok(())
}

// Apply metadata settings: instance ID, hostname
pub fn apply(metadata: &Metadata) -> Result<()> {
    // Persist instance ID
    write_instance_id(&metadata.instance_id)?;

    // Settting hostname
    set_hostname(&metadata.hostname)?;

    // Update hosts file
    update_hosts_file(&metadata.hostname)?;

    Ok(())
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_apply_instance_id() {
        let temp_dir = TempDir::new().unwrap();
        let instance_id_path = temp_dir.path().join("instance-id");
        let instance_id = "test-instance-123";
        write_instance_id_with_path(instance_id, instance_id_path.to_str().unwrap()).unwrap();

        assert_eq!(fs::read_to_string(&instance_id_path).unwrap(), instance_id);
    }

    #[test]
    fn test_write_hostname_file() {
        let temp_dir = TempDir::new().unwrap();
        let hostname_path = temp_dir.path().join("hostname");
        let hostname = "test-host";
        write_hostname_file(hostname, hostname_path.to_str().unwrap()).unwrap();
        let content = fs::read_to_string(&hostname_path).unwrap();
        assert_eq!(content, hostname);
    }

    #[test]
    fn test_update_hosts_file() {
        let temp_dir = TempDir::new().unwrap();
        let hostname = "test-host";
        let hosts_path = temp_dir.path().join("hosts");
        update_hosts_file_with_path(hostname, hosts_path.to_str().unwrap()).unwrap();
        let content = fs::read_to_string(&hosts_path).unwrap();
        assert!(content.contains(&format!("127.0.1.1 {}", hostname)));
    }
}
