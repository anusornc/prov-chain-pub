//! Data Visibility Control (Encryption)
//!
//! This module implements ChaCha20-Poly1305 encryption for private RDF triples.
//! It allows participants to store data on the public chain that is only readable
//! by holders of the decryption key.

use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext (hex encoded)
    pub ciphertext: String,
    /// The nonce used for encryption (hex encoded)
    pub nonce: String,
    /// ID of the key needed to decrypt (e.g., a specific participant ID or group ID)
    pub key_id: String,
}

/// Manages encryption and decryption
pub struct PrivacyManager;

impl PrivacyManager {
    /// Encrypt data using a shared secret key
    pub fn encrypt(data: &str, key: &[u8; 32], key_id: &str) -> Result<EncryptedData> {
        let cipher = ChaCha20Poly1305::new(key.into());
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes); // 96-bits; unique per message

        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            ciphertext: hex::encode(ciphertext),
            nonce: hex::encode(nonce_bytes),
            key_id: key_id.to_string(),
        })
    }

    /// Decrypt data using a shared secret key
    pub fn decrypt(encrypted: &EncryptedData, key: &[u8; 32]) -> Result<String> {
        let cipher = ChaCha20Poly1305::new(key.into());
        
        let nonce_bytes = hex::decode(&encrypted.nonce)
            .map_err(|_| anyhow!("Invalid nonce hex"))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext_bytes = hex::decode(&encrypted.ciphertext)
            .map_err(|_| anyhow!("Invalid ciphertext hex"))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext_bytes.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Invalid UTF-8 in decrypted data: {}", e))
    }
    
    /// Generate a new random 32-byte key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = PrivacyManager::generate_key();
        let data = "Sensitive Supply Chain Data: Batch #123 contains strict IP.";
        
        let encrypted = PrivacyManager::encrypt(data, &key, "group_a")
            .expect("Encryption failed");
            
        assert_ne!(encrypted.ciphertext, data);
        
        let decrypted = PrivacyManager::decrypt(&encrypted, &key)
            .expect("Decryption failed");
            
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_decryption_with_wrong_key() {
        let key1 = PrivacyManager::generate_key();
        let key2 = PrivacyManager::generate_key();
        let data = "Sensitive Data";
        
        let encrypted = PrivacyManager::encrypt(data, &key1, "group_a")
            .unwrap();
            
        let result = PrivacyManager::decrypt(&encrypted, &key2);
        assert!(result.is_err(), "Decryption should fail with wrong key");
    }
}
