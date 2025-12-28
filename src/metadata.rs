use crate::config::Metadata;
use libc::{SYS_sethostname, syscall};
use std::ffi::CString;
use std::io;

pub fn apply(metadata: &Metadata) -> io::Result<()> {
    // Persist instance ID
    write_instance_id(&metadata.instance_id, "/var/lib/hero-init/instance-id")?;

    // Set hostname using libc syscall
    let cstr = CString::new(metadata.hostname.as_str()).unwrap();
    let res = unsafe { syscall(SYS_sethostname, cstr.as_ptr(), metadata.hostname.len()) };

    if res == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
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

// Unit tests
#[cfg(test)]
mod tests {
    use std::fs;

    use crate::config;
    use crate::metadata;
    use tempfile::TempDir;
    #[test]
    fn test_apply_metadata() {
        let temp_dir = TempDir::new().unwrap();

        let cfg = config::Configuration {
            metadata: crate::config::Metadata {
                hostname: "test-host".to_string(),
                instance_id: "instance-123".to_string(),
            },
            network: vec![config::Ethernet::default()],
            users: vec![config::User::default()],
            mounts: vec![config::Mount::default()],
            extension: config::Extension::default(),
        };
        let instance_id_path = temp_dir.path().join("instance-id");

        metadata::write_instance_id(
            &cfg.metadata.instance_id,
            instance_id_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(&instance_id_path).unwrap(),
            cfg.metadata.instance_id
        );
    }
}
