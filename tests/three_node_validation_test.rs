use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn three_node_validation_test() {
    // 1. Setup Test Environment
    let authority_dir = tempdir().unwrap();
    let node2_dir = tempdir().unwrap();
    let node3_dir = tempdir().unwrap();

    let authority_port = 8080;
    let node2_port = 8081;
    let node3_port = 8082;

    // 2. Configure and Launch Nodes
    // Authority Node
    let authority_config_path = authority_dir.path().join("config.toml");
    let mut authority_config = File::create(&authority_config_path).unwrap();
    write!(
        authority_config,
        r#"[network]
listen_port = {}
network_id = "test-network"

[consensus]
is_authority = true

[storage]
data_dir = "./"
"#,
        authority_port
    )
    .unwrap();

    let mut authority_node = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(authority_dir.path())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to start authority node");

    // Give the authority node a moment to start
    thread::sleep(Duration::from_secs(10));

    // Regular Node 2
    let node2_config_path = node2_dir.path().join("config.toml");
    let mut node2_config = File::create(&node2_config_path).unwrap();
    write!(
        node2_config,
        r#"[network]
listen_port = {}
network_id = "test-network"
known_peers = ["127.0.0.1:{}"]

[consensus]
is_authority = false

[storage]
data_dir = "./"
"#,
        node2_port, authority_port
    )
    .unwrap();

    let mut node2 = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(node2_dir.path())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to start node 2");

    // Regular Node 3
    let node3_config_path = node3_dir.path().join("config.toml");
    let mut node3_config = File::create(&node3_config_path).unwrap();
    write!(
        node3_config,
        r#"[network]
listen_port = {}
network_id = "test-network"
known_peers = ["127.0.0.1:{}"]

[consensus]
is_authority = false

[storage]
data_dir = "./"
"#,
        node3_port, authority_port
    )
    .unwrap();

    let mut node3 = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(node3_dir.path())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to start node 3");

    // Give the regular nodes a moment to start and connect
    thread::sleep(Duration::from_secs(20));

    // 3. Add a Block and Verify Synchronization
    let test_data_path = authority_dir.path().join("test_data.ttl");
    let mut test_data = File::create(&test_data_path).unwrap();
    write!(
        test_data,
        r#"@prefix ex: <http://example.org/> .
ex:subject ex:predicate ex:object ."#
    )
    .unwrap();

    let add_block_output = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(authority_dir.path())
    .args(["add-file", test_data_path.to_str().unwrap()])
    .output()
    .expect("Failed to add block");

    assert!(add_block_output.status.success());

    // Wait for synchronization
    thread::sleep(Duration::from_secs(10));

    // 4. Validate Blockchain Integrity
    for dir in &[&authority_dir, &node2_dir, &node3_dir] {
        let validate_output = std::process::Command::new(
            "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
        )
        .current_dir(dir.path())
        .arg("validate")
        .output()
        .expect("Failed to validate node");
        assert!(validate_output.status.success());
    }

    let authority_dump_output = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(authority_dir.path())
    .arg("dump")
    .output()
    .expect("Failed to dump authority node");
    let authority_dump_str = String::from_utf8_lossy(&authority_dump_output.stdout);
    let authority_dump: Vec<Value> = authority_dump_str
        .split('}')
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect();

    let node2_dump_output = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(node2_dir.path())
    .arg("dump")
    .output()
    .expect("Failed to dump node 2");
    let node2_dump_str = String::from_utf8_lossy(&node2_dump_output.stdout);
    let node2_dump: Vec<Value> = node2_dump_str
        .split('}')
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect();

    let node3_dump_output = std::process::Command::new(
        "/Users/anusornchaikaew/Work/Phd/ProvChainOrg/target/debug/provchain-org",
    )
    .current_dir(node3_dir.path())
    .arg("dump")
    .output()
    .expect("Failed to dump node 3");
    let node3_dump_str = String::from_utf8_lossy(&node3_dump_output.stdout);
    let node3_dump: Vec<Value> = node3_dump_str
        .split('}')
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect();

    for i in 0..authority_dump.len() {
        let authority_block = &authority_dump[i];
        let node2_block = &node2_dump[i];
        let node3_block = &node3_dump[i];

        assert_eq!(authority_block["index"], node2_block["index"]);
        assert_eq!(authority_block["index"], node3_block["index"]);

        assert_eq!(authority_block["data"], node2_block["data"]);
        assert_eq!(authority_block["data"], node3_block["data"]);

        assert_eq!(
            authority_block["previous_hash"],
            node2_block["previous_hash"]
        );
        assert_eq!(
            authority_block["previous_hash"],
            node3_block["previous_hash"]
        );

        assert_eq!(authority_block["hash"], node2_block["hash"]);
        assert_eq!(authority_block["hash"], node3_block["hash"]);
    }

    // 5. Teardown
    authority_node.kill().unwrap();
    node2.kill().unwrap();
    node3.kill().unwrap();
}
