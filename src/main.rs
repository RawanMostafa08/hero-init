use hero_init::*;

use anyhow::Result;
use config::Configuration;
use std::fs;
use std::path::Path;

use simplelog::*;

const DEVICE_LABEL: &str = "SEED";

fn load_config(path: &Path) -> Result<Configuration> {
    let data = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&data)?)
}

fn is_first_boot(cfg: &Configuration) -> Result<bool> {
    match fs::read_to_string(paths::INSTANCE_ID_PATH) {
        Ok(stored) => Ok(stored.trim() != cfg.metadata.instance_id),
        Err(_) => Ok(true), // file doesn't exist, assume first boot
    }
}

fn init_logger() -> Result<()> {
    let log_file = {
        // Try /var/log first
        if let Err(e) = std::fs::create_dir_all(paths::HERO_LOG_DIR_PATH) {
            eprintln!("Could not create {}: {}", paths::HERO_LOG_DIR_PATH, e);
        }

        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(paths::HERO_LOG_PATH)
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!(
                    "Failed to open {}: {}, falling back to {}",
                    paths::HERO_LOG_PATH,
                    e,
                    paths::HERO_LOG_FALLBACK_PATH
                );
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(paths::HERO_LOG_FALLBACK_PATH)?
            }
        }
    };

    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).map_err(|e| {
        eprintln!("Failed to init logger: {}", e);
        anyhow::anyhow!("Logger init failed")
    })?;

    Ok(())
}

fn main() -> Result<()> {
    init_logger()?;

    log::info!("hero-init starting");

    // Discover and mount the SEED device
    let seed_device = discovery::find_seed_device(DEVICE_LABEL).expect("no SEED device found");
    log::info!("Found SEED device at {:?}", seed_device);

    let mount_path = Path::new(paths::MOUNT_PATH);
    discovery::mount_seed(seed_device, mount_path)?;

    // Load configuration from the mounted SEED
    let cfg = load_config(mount_path.join("hero-init.yaml").as_path())?;

    // Load state
    let mut state = state::load_state()?;

    // Apply configurations
    let is_first = is_first_boot(&cfg)?;
    if !is_first {
        log::info!("Subsequent boot, skipping per-instance configuration");
        return Ok(());
    }

    log::info!("First boot detected, applying configuration");

    if !state::is_module_complete(&state, "metadata") {
        metadata::apply(&cfg.metadata)?;
        state::mark_module_complete(&mut state, "metadata");
        state::save_state(&state)?;
    }

    if !state::is_module_complete(&state, "network") {
        network::apply(&cfg.network)?;
        state::mark_module_complete(&mut state, "network");
        state::save_state(&state)?;
    }

    if !state::is_module_complete(&state, "users") {
        users::apply(&cfg.users)?;
        state::mark_module_complete(&mut state, "users");
        state::save_state(&state)?;
    }

    if !state::is_module_complete(&state, "runcmd") && !cfg.runcmd.is_empty() {
        log::info!("Running user commands ({} commands)", cfg.runcmd.len());
        execution::apply(&cfg.runcmd)?;
        state::mark_module_complete(&mut state, "runcmd");
        state::save_state(&state)?;
    }

    log::info!("hero-init completed successfully");
    Ok(())
}
