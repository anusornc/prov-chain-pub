use provchain_org::core::blockchain::Blockchain;
use provchain_org::security::encryption::{EncryptedData, PrivacyManager};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_data_visibility_control() {
    // 1. Setup Blockchain
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));

    // 2. Generate Keys
    let authorized_key = PrivacyManager::generate_key();
    let unauthorized_key = PrivacyManager::generate_key(); // Different key

    // 3. Create Encrypted Transaction
    let sensitive_data = "SECRET_FORMULA_V2";
    let encrypted = PrivacyManager::encrypt(sensitive_data, &authorized_key, "group_admin")
        .expect("Encryption failed");

    let encrypted_json = serde_json::to_string(&encrypted).unwrap();

    // 4. Submit Block with Encrypted Data
    {
        let mut bc = blockchain.write().await;
        let block = bc
            .create_block_proposal(
                "PUBLIC_METADATA_ONLY".to_string(), 
                Some(encrypted_json.clone()),
                "VALIDATOR".to_string()
            )
            .expect("Failed to create block");

        bc.submit_signed_block(block)
            .expect("Failed to submit block");
    }

    // 5. Verify Visibility
    let bc_read = blockchain.read().await;
    let stored_block = bc_read.chain.last().expect("Chain should have block");

    // Scenario A: Unauthorized Access (No key / Wrong key)
    // The user can see the ciphertext but cannot read the message
    let stored_encrypted_json = stored_block
        .encrypted_data
        .as_ref()
        .expect("Should have encrypted data");
    let stored_encrypted: EncryptedData = serde_json::from_str(stored_encrypted_json).unwrap();

    let decrypt_attempt = PrivacyManager::decrypt(&stored_encrypted, &unauthorized_key);
    assert!(
        decrypt_attempt.is_err(),
        "Unauthorized key should NOT decrypt data"
    );

    // Scenario B: Authorized Access (Correct key)
    let decrypted_data = PrivacyManager::decrypt(&stored_encrypted, &authorized_key)
        .expect("Authorized key SHOULD decrypt data");

    assert_eq!(
        decrypted_data, sensitive_data,
        "Decrypted data must match original"
    );
}
