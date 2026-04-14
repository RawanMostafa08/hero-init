# hero-init

hero-init is a system initialization tool designed to configure Linux systems at boot time using a SEED device. It reads configuration from a YAML file on a mounted SEED device and applies various system configurations including network settings, user accounts, hostname, and custom commands.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
  - [Configuration Structure](#configuration-structure)
  - [Metadata Configuration](#metadata-configuration)
  - [Network Configuration](#network-configuration)
  - [User Configuration](#user-configuration)
  - [Run Commands](#run-commands)
- [Module Functionalities](#module-functionalities)
  - [Discovery Module](#discovery-module)
  - [Metadata Module](#metadata-module)
  - [Network Module](#network-module)
  - [Users Module](#users-module)
  - [Execution Module](#execution-module)
  - [State Module](#state-module)
- [Tool Functions](#tool-functions)
- [How to Configure](#how-to-configure)
- [Adding Configuration](#adding-configuration)
- [Seed Partition](#seed-partition)
- [Example Configuration](#example-configuration)

## Overview

hero-init is a Rust-based system initialization tool that runs at boot time to configure a Linux system based on a configuration file stored on a SEED device. The SEED device is a specially labeled storage device that contains the configuration needed to initialize the system.

The tool is designed to be idempotent, meaning it can be run multiple times without causing issues. It tracks which configuration modules have been applied using a state file to prevent re-applying configurations on subsequent boots.

## Features

- Automatic discovery and mounting of SEED device
- Network configuration (static IPs, DHCP, routes, DNS)
- User account management (creation, SSH keys, sudo access)
- Hostname and hosts file configuration
- Custom command execution with environment variables
- Idempotent operation with state tracking
- Comprehensive logging

## How It Works

1. At boot time, hero-init searches for a block device labeled "SEED"
2. The SEED device is mounted read-only
3. Configuration is loaded from `hero-init.yaml` on the mounted device
4. The tool checks if this is the first boot by comparing instance IDs
5. If it's the first boot, configuration modules are applied in order:
   - Metadata (hostname, instance ID)
   - Network configuration
   - User account creation
   - Custom commands execution
6. Each module's completion is tracked in a state file to prevent re-application

## Configuration

### Configuration Structure

The configuration file (`hero-init.yaml`) uses YAML format and contains the following top-level sections:

```yaml
instance_id: "unique-identifier"
hostname: "system-hostname"
network:
  # Network configuration
users:
  - # User configurations
runcmd:
  - # Custom commands
```

### Metadata Configuration

The metadata section defines system identification:

```yaml
instance_id: "hero-001"  # Unique identifier for this instance
hostname: "hero-vm"      # System hostname
```

### Network Configuration

The network section configures network interfaces:

```yaml
network:
  interfaces:
    - name: ens3
      mac: "52:54:00:12:34:56"
      dhcp4: false
      addresses:
        - 10.0.2.100/24
      routes:
        - to: default
          via: 10.0.2.2
      nameservers:
        addresses:
          - 10.0.2.3
          - 8.8.8.8
        search:
          - hero.local
```

### User Configuration

The users section defines user accounts to create:

```yaml
users:
  - name: john
    ssh_authorized_keys:
      - "ssh-ed25519 AAAAC.... john@hero-test"
    groups:
      - sudo
    sudo: true
```

### Run Commands

The runcmd section allows execution of custom commands:

```yaml
runcmd:
  - "echo 'Hello from hero-init' >> /tmp/test1.txt"
  - cmd: "echo $MY_VAR > /tmp/test2.txt"
    env:
      MY_VAR: "190"
```

## Module Functionalities

### Discovery Module

The discovery module is responsible for finding and mounting the SEED device:

- Searches for block devices with the label "SEED" in `/dev/disk/by-label`
- Mounts the device read-only to `/run/hero-init/seed`
- Provides functions for device capacity detection and partition management

Key functions:
- `find_seed_device(label: &str) -> Option<PathBuf>` - Finds a device by label
- `mount_seed(device: PathBuf, target: &Path) -> io::Result<()>` - Mounts the device

### Metadata Module

The metadata module handles system identification configuration:

- Sets the system hostname using both file update and system call
- Writes the instance ID to persistent storage
- Updates the hosts file with the new hostname

Key functions:
- `set_hostname(hostname: &str) -> Result<()>` - Sets system hostname
- `write_instance_id(instance_id: &str) -> Result<()>` - Persists instance ID
- `update_hosts_file(hostname: &str) -> Result<()>` - Updates /etc/hosts

### Network Module

The network module configures network interfaces:

- Configures static IPs, DHCP, routes, and DNS settings
- Directly applies network configuration using ip commands

Key functions:
- `apply(network: &Network) -> Result<()>` - Main network configuration function

### Users Module

The users module manages user accounts:

- Creates system users with specified shells and groups
- Sets up SSH authorized keys for passwordless authentication
- Configures sudo access for users
- Sets proper file permissions and ownership

Key functions:
- `add_system_user(username: &str, shell: &str, groups: &[&str]) -> Result<()>` - Creates a user
- `inject_ssh_keys(username: &str, keys: &[String]) -> Result<()>` - Sets up SSH keys
- `add_sudo_rule(username: &str) -> Result<()>` - Grants sudo access
- `apply(users: &[User]) -> Result<()>` - Main user configuration function

### Execution Module

The execution module handles custom command execution:

- Runs shell commands with optional environment variables
- Provides atomic file writing functionality
- Executes commands in the order specified in configuration

Key functions:
- `run_cmd(command: &str, env_vars: HashMap<String, String>) -> Result<()>` - Executes a command
- `write_file_atomic(path: &Path, content: &str, mode: u32) -> Result<()>` - Writes file atomically
- `apply(commands: &[RunCommand]) -> Result<()>` - Main execution function

### State Module

The state module tracks which configuration modules have been applied:

- Loads and saves state to `/var/lib/hero-init/state`
- Prevents re-application of configurations on subsequent boots
- Uses a YAML file with a set of completed module names

Key functions:
- `load_state() -> Result<HeroState>` - Loads state from disk
- `save_state(state: &HeroState) -> Result<()>` - Saves state to disk
- `is_module_complete(state: &HeroState, module: &str) -> bool` - Checks if module is complete
- `mark_module_complete(state: &mut HeroState, module: &str)` - Marks module as complete

## Tool Functions

hero-init provides several utility functions across its modules:

1. **Device Discovery**: Automatically finds and mounts the SEED device
2. **Configuration Parsing**: Reads and validates YAML configuration
3. **Idempotent Operations**: Ensures configurations are only applied once
4. **State Management**: Tracks completed configuration modules
5. **Error Handling**: Comprehensive error handling with detailed logging
6. **Atomic Operations**: File operations are atomic to prevent corruption

## How to Configure

1. Create a SEED device with a filesystem labeled "SEED"
2. Create a `hero-init.yaml` file on the SEED device with your configuration
3. Boot the system with the SEED device attached
4. hero-init will automatically run and apply the configuration

## Adding Configuration

To add configuration:

1. Mount the SEED device on a running system:
   ```bash
   sudo mkdir -p /mnt/seed
   sudo mount /dev/disk/by-label/SEED /mnt/seed
   ```

2. Edit or create `/mnt/seed/hero-init.yaml` with your desired configuration

3. Unmount the device:
   ```bash
   sudo umount /mnt/seed
   ```

4. Boot the target system with the SEED device attached

## Seed Partition

The SEED partition must:

1. Be a filesystem on a block device
2. Have the label "SEED"
3. Contain a `hero-init.yaml` file with valid configuration
4. Be readable by the system

To create a SEED partition:

1. Create a filesystem with the SEED label:
   ```bash
    dd if=/dev/zero of=seed.img bs=1M count=10
    mkfs.vfat -n SEED seed.img
   ```

3. Mount and populate with configuration:
   ```bash
   sudo mount seed.img /mnt
   # Add hero-init.yaml
   sudo umount /mnt/seed
   ```

## Example Configuration

See [example.yaml](example.yaml) for a complete configuration example:

```yaml
instance_id: "hero-001"
hostname: "hero-vm"
network:
  interfaces:
    - name: ens3
      mac: "52:54:00:12:34:56"
      dhcp4: false
      addresses:
        - 10.0.2.100/24
      routes:
        - to: default
          via: 10.0.2.2
      nameservers:
        addresses:
          - 10.0.2.3
          - 8.8.8.8
        search:
          - hero.local
users:
  - name: john
    ssh_authorized_keys:
      - "ssh-ed25519 AAAAC.... john@hero-test"
    groups:
      - sudo
    sudo: true
runcmd:
  - "echo 'Hello from hero-init' >> /tmp/test1.txt"
  - cmd: "echo $MY_VAR > /tmp/test2.txt"
    env:
      MY_VAR: "190"
