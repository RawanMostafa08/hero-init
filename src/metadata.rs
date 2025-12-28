use crate::config::Metadata;
use libc::{SYS_sethostname, syscall};
use std::ffi::CString;
use std::fs;
use std::io;
use std::io::Write;

pub fn apply(metadata: &Metadata) -> io::Result<()> {
    // Persist instance ID
    write_instance_id(&metadata.instance_id, "/var/lib/hero-init/instance-id")?;

    // Settting hostname
    // 1- writing to /etc/hostname
    write_hostname_file(&metadata.hostname, "/etc/hostname")?;

    // 3- update hosts file
    update_hosts_file(&metadata.hostname, "/etc/hosts")?;

    // 4- using libc syscall
    set_hostname_syscall(&metadata.hostname)?;

    Ok(())
}

fn write_instance_id(instance_id: &str, instance_id_path: &str) -> io::Result<()> {
    use std::fs;
    use std::path::Path;

    // Write persistent instance-id
    if let Some(parent) = Path::new(instance_id_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(instance_id_path, instance_id)?;
    Ok(())
}

fn write_hostname_file(hostname: &str, path: &str) -> io::Result<()> {
    fs::write(path, hostname.as_bytes())
}

fn update_hosts_file(hostname: &str, path: &str) -> io::Result<()> {
    let mut hosts_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    writeln!(hosts_file, "127.0.1.1 {}", hostname)?;
    Ok(())
}

fn set_hostname_syscall(hostname: &str) -> io::Result<()> {
    let cstr = CString::new(hostname).unwrap();
    let res = unsafe { syscall(SYS_sethostname, cstr.as_ptr(), hostname.len()) };

    if res == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use crate::metadata;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_apply_instance_id() {
        let temp_dir = TempDir::new().unwrap();
        let instance_id_path = temp_dir.path().join("instance-id");
        let instance_id = "test-instance-123";
        metadata::write_instance_id(instance_id, instance_id_path.to_str().unwrap()).unwrap();

        assert_eq!(fs::read_to_string(&instance_id_path).unwrap(), instance_id);
    }

    #[test]
    fn test_write_hostname_file() {
        let temp_dir = TempDir::new().unwrap();
        let hostname_path = temp_dir.path().join("hostname");
        let hostname = "test-host";
        metadata::write_hostname_file(hostname, hostname_path.to_str().unwrap()).unwrap();
        let content = fs::read_to_string(&hostname_path).unwrap();
        assert_eq!(content, hostname);
    }

    #[test]
    fn test_update_hosts_file() {
        let temp_dir = TempDir::new().unwrap();
        let hostname = "test-host";
        let hosts_path = temp_dir.path().join("hosts");
        metadata::update_hosts_file(hostname, hosts_path.to_str().unwrap()).unwrap();
        let content = fs::read_to_string(&hosts_path).unwrap();
        assert!(content.contains(&format!("127.0.1.1 {}", hostname)));
    }
}
