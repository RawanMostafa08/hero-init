use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const SECTOR_SIZE: u64 = 512;

// Search for a block device with the given label in /dev/disk/by-label
pub fn find_seed_device(label: &str) -> Option<PathBuf> {
    let by_label = Path::new("/dev/disk/by-label");

    let entries = fs::read_dir(by_label).ok()?;

    for entry in entries.flatten() {
        if entry.file_name() == label {
            // symlink -> real block device (e.g., /dev/sda1)
            return fs::read_link(entry.path()).ok();
        }
    }
    None
}

// Mount the seed device to the target path
pub fn mount_seed(device: PathBuf, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;

    let status = Command::new("mount")
        .args([
            "-o",
            "ro", // read-only
            device.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("failed to mount seed device"))
    }
}

// Get the disk capacity in bytes of the given block device
pub fn get_disk_capacity(device: &Path) -> io::Result<u64> {
    let name = device
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid device"))?;

    let size_path = Path::new("/sys/class/block").join(name).join("size");

    let sectors: u64 = fs::read_to_string(size_path)?
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid size"))?;

    Ok(sectors * SECTOR_SIZE)
}

// Create a partition table on the given device with the specified label type
pub fn create_partition_table(device: &Path, label_type: &str) -> io::Result<()> {
    // e.g.,  parted -s /dev/sdb mklabel gpt

    let status = Command::new("parted")
        .args([
            "-s", // script mode: no interactive prompts
            device.to_str().unwrap(),
            "mklabel",
            label_type, // gpt or msdos
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("failed to create partition table"))
    }
}

// Format the given partition with the specified filesystem type and label
pub fn format_partition(partition: &Path, fs_type: &str, label: &str) -> io::Result<()> {
    // e.g., mkfs.ext4 -L SEED /dev/sdb1

    let mkfs = format!("mkfs.{}", fs_type);

    let status = Command::new(mkfs)
        .args(["-L", label, partition.to_str().unwrap()])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("failed to format partition"))
    }
}
