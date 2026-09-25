//! Canonical length-prefixing and domain-separated hashing for network contracts.

use sha2::{Digest, Sha256};

/// Append one UTF-8 field with an unsigned 64-bit byte-length prefix.
pub(super) fn write_len_prefixed_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// Hash length-prefixed parts under an explicit protocol-domain label.
pub(super) fn hash_domain_separated_parts(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}
