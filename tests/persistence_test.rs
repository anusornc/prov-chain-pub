use provchain_org::wallet::{Participant, WalletManager};
use tempfile::tempdir;

#[test]
fn test_wallet_persistence_with_master_key() {
    let temp_dir = tempdir().unwrap();
    let storage_dir = temp_dir.path().join("wallets");
    let master_key_path = temp_dir.path().join("master.key");

    let participant_id;
    let participant_name = "Persistent Participant";

    // 1. Initialize WalletManager (this should create the master key)
    {
        let mut manager = WalletManager::new(&storage_dir, Some(master_key_path.clone())).unwrap();
        
        let participant = Participant::new_farmer(
            participant_name.to_string(),
            "Persistence Test Location".to_string(),
        );
        participant_id = manager.create_wallet(participant).unwrap();
        
        assert!(manager.get_wallet(participant_id).is_some());
    }

    // 2. Drop manager (simulating shutdown) and check files exist
    assert!(master_key_path.exists());
    assert!(storage_dir.exists());

    // 3. Re-initialize WalletManager using the same master key
    {
        let mut manager = WalletManager::new(&storage_dir, Some(master_key_path.clone())).unwrap();
        
        // Load the wallet from disk
        manager.load_wallet(participant_id).unwrap();
        
        let wallet = manager.get_wallet(participant_id).unwrap();
        assert_eq!(wallet.participant.name, participant_name);
        
        // Verify we can decrypt the private key (proof that encryption key is correct)
        // Note: Wallet.signing_key is transient in memory struct but recreated from file content if decryption worked
        assert!(wallet.signing_key.is_some());
    }
}
