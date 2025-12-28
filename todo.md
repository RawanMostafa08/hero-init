## Discovery & Hardware Primitives
Functions that interact with the kernel and `/dev` tree to find configuration sources.

- [ ] **find_seed_device(label: &str) -> Option<PathBuf>**  
  Scans `/dev/disk/by-label/` for the `SEED` volume to locate user-data/meta-data files.

- [ ] **mount_seed(device: PathBuf, target: &Path) -> Result<()>**  
  to attach the FAT32/ISO9660 seed to a temporary path.

- [ ] **get_disk_capacity(device: &Path) -> u64**  
  reads `/sys/class/block/` to get total bytes of a target disk.

- [ ] **create_partition_table(device: &Path, label_type: &str)**  
  initialize a disk (GPT/MBR).

- [ ] **format_partition(partition: &Path, fs_type: &str, label: &str)**  
  Executes `mkfs.{fs_type}` with the provided label.

## Identity & System Primitives
Functions that modify the core system state to match cloud-init style configuration.

- [ ] **set_hostname(name: &str) -> Result<()>**  
  Overwrites `/etc/hostname` and calls the `sethostname` syscall.

- [ ] **update_hosts_file(hostname: &str)**  
  Appends `127.0.1.1 <hostname>` to `/etc/hosts` to prevent resolution errors.

- [ ] **write_network_config(config_yaml: &str, provider: &str)**  
  Writes a file to `/etc/netplan/01-hero-init.yaml` or `/etc/network/interfaces`.

- [ ] **apply_network_config()**  
  Executes `netplan apply` or `systemctl restart networking`.

## User & Security Primitives
Functions that handle user creation and SSH key injection.

- [ ] **add_system_user(username: &str, shell: &str, groups: Vec<&str>)**  
  Wraps `useradd -m -s {shell} -G {groups} {username}`.

- [ ] **inject_ssh_keys(username: &str, keys: Vec<String>)**  
  Resolves user home directory via `/etc/passwd`, creates `.ssh/` folder, and writes keys to `authorized_keys`.

- [ ] **set_permissions(path: &Path, mode: u32, uid: u32, gid: u32)**  
  Calls `chmod` and `chown` to secure the `.ssh` folder.

- [ ] **add_sudo_rule(username: &str)**  
  Creates `/etc/sudoers.d/{username}` with `ALL=(ALL) NOPASSWD:ALL`.

## Logic & Execution Primitives
Functions that handle `RunCmd` and customization requirements.

- [ ] **run_cmd(command: &str, env_vars: HashMap<String, String>)**  
  to execute strings in a subshell.

- [ ] **write_file_atomic(path: &Path, content: &str, mode: u32)**  
  Writes to a `.tmp` file and moves it to the target path to prevent corruption.

- [ ] **mount_fstab_entry(device: &Path, mount_point: &Path, fs_type: &str)**  
  Appends a line to `/etc/fstab` and executes `mount -a`.

## Persistence & State Primitives
Functions that ensure HeroInit runs idempotently.

- [ ] **load_state(path: &Path) -> HeroState**  
  Reads a JSON/YAML file from `/var/lib/cloudinit/state`.

- [ ] **save_state(path: &Path, state: &HeroState)**  
  Serializes the current progress to disk.

- [ ] **is_module_complete(module: &str) -> bool**  
  Checks the state object to see if `disk_setup` or `user_setup` has already run.
