use std::fs;

use hero_init::config;
use hero_init::metadata;
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
