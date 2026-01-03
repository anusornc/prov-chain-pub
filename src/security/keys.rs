//! Cryptographically secure key generation utilities
//!
//! This module provides secure key generation functions that use
//! cryptographically secure random number generators (CSPRNG).

use anyhow::Result;
use ed25519_dalek::SigningKey;
use rand::RngCore;
use rand::rngs::OsRng;

/// Generate a new Ed25519 signing key using cryptographically secure RNG
///
/// # Security
/// Uses OsRng which is backed by the operating system's CSPRNG:
/// - Linux: getrandom(2) syscall
/// - macOS: getentropy(2)
/// - Windows: BCryptGenRandom
///
/// # Example
/// ```no_run
/// use provchain_org::security::keys::generate_signing_key;
///
/// # fn main() -> anyhow::Result<()> {
/// let keypair = generate_signing_key()?;
/// let public_key = keypair.verifying_key();
/// # Ok(())
/// # }
/// ```
pub fn generate_signing_key() -> Result<SigningKey> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    Ok(SigningKey::from_bytes(&bytes))
}

/// Generate a new Ed25519 signing key with a specific seed (TESTING ONLY)
///
/// # Security Warning
/// This function should ONLY be used in testing environments.
/// Never use deterministic keys in production.
#[cfg(test)]
pub fn generate_signing_key_with_seed(seed: &[u8; 32]) -> SigningKey {
    SigningKey::from_bytes(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_signing_key() {
        let key1 = generate_signing_key().unwrap();
        let key2 = generate_signing_key().unwrap();

        // Keys should be different
        assert_ne!(key1.to_bytes(), key2.to_bytes());
    }

    #[test]
    fn test_deterministic_key_generation() {
        let seed = [42u8; 32];
        let key1 = generate_signing_key_with_seed(&seed);
        let key2 = generate_signing_key_with_seed(&seed);

        // Deterministic keys should be identical
        assert_eq!(key1.to_bytes(), key2.to_bytes());
    }
}
