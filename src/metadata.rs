use crate::config::Configuration;
use libc::{SYS_sethostname, syscall};
use std::ffi::CString;
use std::io;

pub fn apply(cfg: &Configuration) -> io::Result<()> {
    // Persist instance ID
    write_instance_id(&cfg.metadata.instance_id, "/var/lib/hero-init/instance-id")?;

    // Set hostname using libc syscall
    let cstr = CString::new(cfg.metadata.hostname.as_str()).unwrap();
    let res = unsafe { syscall(SYS_sethostname, cstr.as_ptr(), cfg.metadata.hostname.len()) };

    if res == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub fn write_instance_id(instance_id: &str, instance_id_path: &str) -> io::Result<()> {
    use std::fs;
    use std::path::Path;

    // Write persistent instance-id
    if let Some(parent) = Path::new(instance_id_path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(instance_id_path, instance_id)?;
    Ok(())
}
