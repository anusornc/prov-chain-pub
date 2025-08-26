use provchain_org::core::blockchain::Blockchain;

#[test]
fn test_blockchain_add_and_validate() {
    let mut bc = Blockchain::new();
    let _ = bc.add_block("test data".into());
    let _ = bc.add_block("more test data".into());

    assert!(bc.is_valid(), "Blockchain should be valid after adding blocks");
}

#[test]
fn test_blockchain_detect_tampering() {
    let mut bc = Blockchain::new();
    let _ = bc.add_block("original".into());
    let _ = bc.add_block("another block".into());

    // Tamper with a block's data
    bc.chain[1].data = "tampered".into();

    assert!(!bc.is_valid(), "Blockchain should be invalid after tampering with data");
}

#[test]
fn test_blockchain_dump() {
    let mut bc = Blockchain::new();
    let _ = bc.add_block("data for dump".into());
    let dump_output = bc.dump().expect("Dump should succeed");

    // Basic check to see if the output is valid JSON and contains the data
    assert!(dump_output.starts_with("["));
    assert!(dump_output.contains("data for dump"));
    assert!(dump_output.ends_with("]"));
}

#[test]
fn test_hash_is_different_for_different_data() {
    let mut bc1 = Blockchain::new();
    let _ = bc1.add_block("data1".into());

    let mut bc2 = Blockchain::new();
    let _ = bc2.add_block("data2".into());

    assert_ne!(bc1.chain[1].hash, bc2.chain[1].hash, "Hashes should be different for different data");
}
