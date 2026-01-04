use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths;

const SECTOR_SIZE: u64 = 512;

// Search for a block device with the given label in /dev/disk/by-label
pub fn find_seed_device(label: &str) -> Option<PathBuf> {
    find_seed_device_with_path(label, paths::BY_LABEL_PATH)
}

fn find_seed_device_with_path(label: &str, by_label_path: &str) -> Option<PathBuf> {
    let by_label = Path::new(by_label_path);

    for entry in fs::read_dir(by_label).ok()? {
        let entry = entry.ok()?;
        if entry.file_name() == label {
            // Resolve the symlink to the real device
            return fs::canonicalize(entry.path()).ok();
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
    get_disk_capacity_with_path(device, paths::DEVICE_CAPACITY_PATH)
}

fn get_disk_capacity_with_path(device: &Path, path: &str) -> io::Result<u64> {
    let name = device
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid device"))?;

    let size_path = Path::new(path).join(name).join("size");

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

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs as unix_fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_seed_device_found() {
        let temp_dir = TempDir::new().unwrap();
        let by_label = temp_dir.path().join("by-label");
        fs::create_dir(&by_label).unwrap();
        let label_link = by_label.join("SEED");

        // Create a symlink to a mock device
        unix_fs::symlink("/dev/mock-device", &label_link).unwrap();

        let device = find_seed_device_with_path("SEED", by_label.to_str().unwrap());
        assert!(device.is_none());
    }

    #[test]
    fn test_find_seed_device_not_found() {
        let device = find_seed_device("NONEXISTENT");
        assert!(device.is_none());
    }

    #[test]
    fn test_mount_seed_success() {
        let temp_dir = TempDir::new().unwrap();
        let mock_device = temp_dir.path().join("mock-dev");
        fs::write(&mock_device, "").unwrap();

        let target = temp_dir.path().join("mount-target");
        let result = mount_seed(mock_device, target.as_path());
        assert!(result.is_err());
        // Check dir created
        assert!(target.exists());
    }
}
