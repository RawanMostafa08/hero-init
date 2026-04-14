use anyhow::Result;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::config::RunCommand;

// Run a shell command with specified environment variables
pub fn run_cmd(command: &str, env_vars: HashMap<String, String>) -> Result<()> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let status = cmd.status()?;

    if !status.success() {
        Err(anyhow::anyhow!("failed to execute command"))
    } else {
        Ok(())
    }
}

// Write content to a file atomically with specified permissions
pub fn write_file_atomic(path: &Path, content: &str, mode: u32) -> Result<()> {
    let temp_path = path.with_extension("tmp");

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    fs::set_permissions(&temp_path, fs::Permissions::from_mode(mode))?;
    fs::rename(temp_path, path)?;

    Ok(())
}

pub fn apply(commands: &[RunCommand]) -> Result<()> {
    for (i, cmd) in commands.iter().enumerate() {
        log::info!("Executing command {}/{}", i + 1, commands.len());
        match cmd {
            RunCommand::Simple(command) => {
                log::info!("Running: {}", command);
                if let Err(e) = run_cmd(command, HashMap::new()) {
                    log::error!("Failed to run simple command '{}': {}", command, e);
                    return Err(e);
                }
            }
            RunCommand::Structured { cmd, env } => {
                log::info!("Running: {} (with {} env vars)", cmd, env.len());
                if let Err(e) = run_cmd(cmd, env.clone()) {
                    log::error!("Failed to run structured command '{}': {}", cmd, e);
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_run_cmd_simple() {
        let command = "echo 'test output'";
        let result = run_cmd(command, HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_cmd_with_env() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "hello".to_string());
        let command = "echo $TEST_VAR";
        let result = run_cmd(command, env_vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_file_atomic() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test.txt");
        let content = "Hello, world!";
        let result = write_file_atomic(&test_path, content, 0o644);
        assert!(result.is_ok());
        let written_content = fs::read_to_string(&test_path).unwrap();
        assert_eq!(written_content, content);
        let metadata = fs::metadata(&test_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o644);
    }

    #[test]
    fn test_apply_simple_command() {
        let commands = vec![RunCommand::Simple("echo 'applied'".to_string())];
        let result = apply(&commands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_structured_command() {
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "value".to_string());
        let commands = vec![RunCommand::Structured {
            cmd: "echo $KEY".to_string(),
            env,
        }];
        let result = apply(&commands);
        assert!(result.is_ok());
    }
}
