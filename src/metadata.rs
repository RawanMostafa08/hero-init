use crate::config::Configuration;
use libc::{SYS_sethostname, syscall};
use std::ffi::CString;
use std::io;

pub fn apply(cfg: &Configuration) -> io::Result<()> {
    // Set hostname using libc syscall
    let cstr = CString::new(cfg.metadata.hostname.as_str()).unwrap();
    let res = unsafe { syscall(SYS_sethostname, cstr.as_ptr(), cfg.metadata.hostname.len()) };

    if res == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
