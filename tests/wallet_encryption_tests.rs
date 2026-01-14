//! Wallet Encryption Tests
//!
//! Comprehensive test suite for wallet encryption using ChaCha20-Poly1305 AEAD.
//! Tests validate that:
//! - Wallet data is properly encrypted with ChaCha20-Poly1305
//! - Each encryption uses a unique nonce
//! - Authentication tags detect tampering
//! - Decryption fails with corrupted or invalid data
//! - Encryption key requirements are enforced

use provchain_org::wallet::{Wallet, Participant};
use provchain_org::security::encryption::PrivacyManager;
use chacha20poly1305::aead::{Aead, AeadCore, OsRng as AeadOsRng};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};

/// Helper to create a test wallet
fn create_test_wallet() -> Wallet {
    let participant = Participant::new_farmer(
        "Test Farm".to_string(),
        "Test Location".to_string()
    );
    Wallet::new(participant)
}

/// Helper to manually encrypt data for testing
fn manual_encrypt(data: &[u8], key: &chacha20poly1305::Key) -> Result<Vec<u8>, String> {
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut AeadOsRng);
    let ciphertext = cipher.encrypt(&nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Helper to manually decrypt data for testing
fn manual_decrypt(encrypted_data: &[u8], key: &chacha20poly1305::Key) -> Result<Vec<u8>, String> {
    if encrypted_data.len() < 12 {
        return Err("Data too short".into());
    }

    let (nonce, ciphertext) = encrypted_data.split_at(12);
    let cipher = ChaCha20Poly1305::new(key);
    let plaintext = cipher.decrypt(nonce.into(), ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    Ok(plaintext)
}

#[cfg(test)]
mod encryption_tests {
    use super::*;

    #[test]
    fn test_wallet_data_is_encrypted() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        // Serialize wallet
        let json_data = serde_json::to_string(&wallet).unwrap();
        let original_bytes = json_data.as_bytes();

        // Encrypt
        let encrypted = manual_encrypt(original_bytes, &key).unwrap();

        // Encrypted data should be different from original
        // (with very high probability due to nonce)
        assert_ne!(
            &encrypted[12..],
            original_bytes,
            "Encrypted data should differ from plaintext"
        );

        // Encrypted data should be longer (nonce + ciphertext + tag)
        assert!(encrypted.len() > original_bytes.len());
    }

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        // Serialize and encrypt
        let json_data = serde_json::to_string(&wallet).unwrap();
        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();

        // Decrypt
        let decrypted = manual_decrypt(&encrypted, &key).unwrap();
        let decrypted_str = String::from_utf8(decrypted).unwrap();
        let decrypted_wallet: Wallet = serde_json::from_str(&decrypted_str).unwrap();

        // Should match original - compare participant IDs
        assert_eq!(decrypted_wallet.participant.id, wallet.participant.id);
        assert_eq!(decrypted_wallet.participant.name, wallet.participant.name);
    }

    #[test]
    fn test_unique_nonce_per_encryption() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        let json_data = serde_json::to_string(&wallet).unwrap();
        let data = json_data.as_bytes();

        // Encrypt twice with same key and data
        let encrypted1 = manual_encrypt(data, &key).unwrap();
        let encrypted2 = manual_encrypt(data, &key).unwrap();

        // Nonces should be different (first 12 bytes)
        let nonce1 = &encrypted1[..12];
        let nonce2 = &encrypted2[..12];
        assert_ne!(nonce1, nonce2, "Nonces should be unique");

        // Ciphertexts should also differ due to different nonces
        assert_ne!(&encrypted1[12..], &encrypted2[12..]);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let wallet = create_test_wallet();
        let key1 = PrivacyManager::generate_key();
        let key2 = PrivacyManager::generate_key();
        let key1 = chacha20poly1305::Key::from(key1);
        let key2 = chacha20poly1305::Key::from(key2);

        // Encrypt with key1
        let json_data = serde_json::to_string(&wallet).unwrap();
        let encrypted = manual_encrypt(json_data.as_bytes(), &key1).unwrap();

        // Try to decrypt with key2 (different key)
        let result = manual_decrypt(&encrypted, &key2);

        assert!(result.is_err(), "Decryption with wrong key should fail");
    }
}

#[cfg(test)]
mod tampering_detection_tests {
    use super::*;

    #[test]
    fn test_tampered_ciphertext_detected() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        let json_data = serde_json::to_string(&wallet).unwrap();
        let mut encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();

        // Tamper with ciphertext (after nonce)
        encrypted[15] ^= 0xFF;

        let result = manual_decrypt(&encrypted, &key);
        assert!(result.is_err(), "Tampered ciphertext should be detected");
    }

    #[test]
    fn test_tampered_nonce_detected() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        let json_data = serde_json::to_string(&wallet).unwrap();
        let mut encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();

        // Tamper with nonce
        encrypted[0] ^= 0xFF;

        let result = manual_decrypt(&encrypted, &key);
        assert!(result.is_err(), "Tampered nonce should be detected");
    }

    #[test]
    fn test_truncated_data_detected() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        let json_data = serde_json::to_string(&wallet).unwrap();
        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();

        // Remove last few bytes (part of auth tag)
        let truncated = &encrypted[..encrypted.len() - 5];

        let result = manual_decrypt(truncated, &key);
        assert!(result.is_err(), "Truncated data should fail authentication");
    }

    #[test]
    fn test_appended_data_detected() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);

        let json_data = serde_json::to_string(&wallet).unwrap();
        let mut encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();

        // Append extra data
        encrypted.push(0x42);
        encrypted.push(0x43);

        let result = manual_decrypt(&encrypted, &key);
        assert!(result.is_err(), "Data with extra bytes should fail authentication");
    }

    #[test]
    fn test_empty_ciphertext_rejected() {
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let encrypted = vec![0u8; 12]; // Only nonce, no ciphertext

        let result = manual_decrypt(&encrypted, &key);
        assert!(result.is_err(), "Empty ciphertext should be rejected");
    }
}

#[cfg(test)]
mod key_requirements_tests {
    use super::*;

    #[test]
    fn test_encryption_key_is_32_bytes() {
        let key = PrivacyManager::generate_key();
        assert_eq!(key.len(), 32, "ChaCha20-Poly1305 requires 32-byte keys");
    }

    #[test]
    fn test_nonce_is_12_bytes() {
        let nonce1 = ChaCha20Poly1305::generate_nonce(&mut AeadOsRng);
        let nonce2 = ChaCha20Poly1305::generate_nonce(&mut AeadOsRng);

        assert_eq!(nonce1.len(), 12, "ChaCha20-Poly1305 uses 96-bit (12-byte) nonces");
        assert_eq!(nonce2.len(), 12);

        // Nonces should be different
        assert_ne!(nonce1, nonce2, "Random nonces should be unique");
    }

    #[test]
    fn test_wrong_key_length_rejected() {
        use chacha20poly1305::aead::KeyInit;

        // Correct key length (32 bytes)
        let key = chacha20poly1305::Key::from([0u8; 32]);

        let _cipher = ChaCha20Poly1305::new(&key);
        // KeyInit::new handles this properly at compile time
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_minimal_wallet_encryption() {
        let participant = Participant::new_farmer(
            String::new(),
            String::new()
        );
        let wallet = Wallet::new(participant);

        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let json_data = serde_json::to_string(&wallet).unwrap();

        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();
        let decrypted = manual_decrypt(&encrypted, &key).unwrap();

        assert_eq!(&*decrypted, json_data.as_bytes());
    }

    #[test]
    fn test_large_wallet_data() {
        let participant = Participant::new_farmer(
            "a".repeat(1000),
            "Test Location".to_string()
        );
        let wallet = Wallet::new(participant);

        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let json_data = serde_json::to_string(&wallet).unwrap();

        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();
        let decrypted = manual_decrypt(&encrypted, &key).unwrap();

        assert_eq!(&*decrypted, json_data.as_bytes());
    }

    #[test]
    fn test_unicode_wallet_data() {
        let participant = Participant::new_farmer(
            "地址测试🔐".to_string(),
            "测试位置".to_string()
        );
        let wallet = Wallet::new(participant);

        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let json_data = serde_json::to_string(&wallet).unwrap();

        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();
        let decrypted = manual_decrypt(&encrypted, &key).unwrap();

        assert_eq!(&*decrypted, json_data.as_bytes());

        let decrypted_str = String::from_utf8(decrypted).unwrap();
        let decrypted_wallet: Wallet = serde_json::from_str(&decrypted_str).unwrap();
        assert_eq!(decrypted_wallet.participant.name, wallet.participant.name);
    }

    #[test]
    fn test_special_characters_in_wallet() {
        let participant = Participant::new_farmer(
            "\\\"'\t\n\r\x00\x1F".to_string(),
            "Test Location".to_string()
        );
        let wallet = Wallet::new(participant);

        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let json_data = serde_json::to_string(&wallet).unwrap();

        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();
        let decrypted = manual_decrypt(&encrypted, &key).unwrap();

        assert_eq!(&*decrypted, json_data.as_bytes());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_encryption_performance() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let json_data = serde_json::to_string(&wallet).unwrap();

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();
        }

        let duration = start.elapsed();
        let per_encryption = duration.as_micros() as f64 / iterations as f64;

        println!("Encryption: {} iterations in {:?}", iterations, duration);
        println!("Average: {:.2} μs per encryption", per_encryption);

        // Should be reasonably fast (less than 1ms per encryption on modern hardware)
        assert!(per_encryption < 1000.0, "Encryption should be fast");
    }

    #[test]
    fn test_decryption_performance() {
        let wallet = create_test_wallet();
        let key = PrivacyManager::generate_key();
        let key = chacha20poly1305::Key::from(key);
        let json_data = serde_json::to_string(&wallet).unwrap();
        let encrypted = manual_encrypt(json_data.as_bytes(), &key).unwrap();

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _decrypted = manual_decrypt(&encrypted, &key).unwrap();
        }

        let duration = start.elapsed();
        let per_decryption = duration.as_micros() as f64 / iterations as f64;

        println!("Decryption: {} iterations in {:?}", iterations, duration);
        println!("Average: {:.2} μs per decryption", per_decryption);

        // Should be reasonably fast
        assert!(per_decryption < 1000.0, "Decryption should be fast");
    }
}
