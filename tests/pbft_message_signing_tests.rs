//! PBFT Consensus Message Signing Tests
//!
//! Comprehensive test suite for PBFT message signing and verification.
//! Tests validate that:
//! - All PBFT messages are properly signed
//! - Signatures can be verified
//! - Invalid signatures are rejected
//! - Message tampering is detected

use provchain_org::network::consensus::PbftMessage;
use provchain_org::core::blockchain::Block;
use provchain_org::security::keys::generate_signing_key;
use ed25519_dalek::SigningKey;
use ed25519_dalek::Signer;
use uuid::Uuid;

fn create_test_block(index: u64, data: &str) -> Block {
    Block {
        index,
        timestamp: chrono::Utc::now().to_rfc3339(),
        data: data.to_string(),
        previous_hash: "prev_hash".to_string(),
        hash: format!("block_hash_{}", index),
        encrypted_data: None,
        state_root: "state_root".to_string(),
        validator: "validator".to_string(),
        signature: "signature".to_string(),
    }
}

#[cfg(test)]
mod message_creation_tests {
    use super::*;

    fn create_test_block(index: u64, data: &str) -> Block {
        Block {
            index,
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: data.to_string(),
            previous_hash: "prev_hash".to_string(),
            hash: format!("block_hash_{}", index),
            encrypted_data: None,
            state_root: "state_root".to_string(),
            validator: "validator".to_string(),
            signature: "signature".to_string(),
        }
    }

    #[test]
    fn test_create_signed_pre_prepare() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test data");
        let block_hash = block.hash.clone();

        let msg = PbftMessage::create_pre_prepare(
            0,
            1,
            block_hash,
            block,
            sender,
            &keypair
        );

        assert!(msg.verify_signature());
        assert_eq!(msg.get_sender(), sender);
    }

    #[test]
    fn test_create_signed_prepare() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block_hash = "test_block_hash".to_string();

        let msg = PbftMessage::create_prepare(0, 1, block_hash, sender, &keypair);

        assert!(msg.verify_signature());
        assert_eq!(msg.get_sender(), sender);
    }

    #[test]
    fn test_create_signed_commit() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block_hash = "test_block_hash".to_string();

        let msg = PbftMessage::create_commit(0, 1, block_hash, sender, &keypair);

        assert!(msg.verify_signature());
        assert_eq!(msg.get_sender(), sender);
    }

    #[test]
    fn test_create_signed_view_change() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_view_change(1, sender, &keypair);

        assert!(msg.verify_signature());
        assert_eq!(msg.get_sender(), sender);
    }
}

#[cfg(test)]
mod signature_verification_tests {
    use super::*;

    #[test]
    fn test_verify_valid_pre_prepare_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let msg = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block,
            sender,
            &keypair
        );

        assert!(msg.verify_signature(), "Valid signature should verify");
    }

    #[test]
    fn test_verify_valid_prepare_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_prepare(
            0,
            1,
            "block_hash".to_string(),
            sender,
            &keypair
        );

        assert!(msg.verify_signature());
    }

    #[test]
    fn test_verify_valid_commit_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_commit(
            0,
            1,
            "block_hash".to_string(),
            sender,
            &keypair
        );

        assert!(msg.verify_signature());
    }

    #[test]
    fn test_verify_valid_view_change_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_view_change(1, sender, &keypair);

        assert!(msg.verify_signature());
    }
}

#[cfg(test)]
mod tamper_detection_tests {
    use super::*;

    #[test]
    fn test_detect_tampered_view_number() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        // Create valid message
        let mut msg = PbftMessage::create_pre_prepare(
            0,  // Original view
            1,
            "hash".to_string(),
            block,
            sender,
            &keypair
        );

        // Tamper with view number
        if let PbftMessage::PrePrepare { ref mut view, .. } = msg {
            *view = 999; // Tampered
        }

        // Signature should no longer verify
        assert!(!msg.verify_signature(), "Tampered message should fail verification");
    }

    #[test]
    fn test_detect_tampered_sequence() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let mut msg = PbftMessage::create_prepare(
            0,
            1,  // Original sequence
            "block_hash".to_string(),
            sender,
            &keypair
        );

        // Tamper with sequence number
        if let PbftMessage::Prepare { ref mut sequence, .. } = msg {
            *sequence = 999; // Tampered
        }

        assert!(!msg.verify_signature());
    }

    #[test]
    fn test_detect_tampered_block_hash() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let mut msg = PbftMessage::create_commit(
            0,
            1,
            "original_hash".to_string(),  // Original hash
            sender,
            &keypair
        );

        // Tamper with block_hash
        if let PbftMessage::Commit { ref mut block_hash, .. } = msg {
            *block_hash = "tampered_hash".to_string();
        }

        assert!(!msg.verify_signature());
    }

    #[test]
    fn test_detect_tampered_sender() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let mut msg = PbftMessage::create_view_change(1, sender, &keypair);

        // Tamper with sender
        if let PbftMessage::ViewChange { ref mut sender, .. } = msg {
            *sender = Uuid::new_v4(); // Different sender
        }

        assert!(!msg.verify_signature());
    }
}

#[cfg(test)]
mod invalid_signature_tests {
    use super::*;
    use ed25519_dalek::{Signature, VerifyingKey};

    fn create_block_with_signature(
        signature: Signature,
        public_key: VerifyingKey,
    ) -> PbftMessage {
        PbftMessage::PrePrepare {
            view: 0,
            sequence: 1,
            block_hash: "hash".to_string(),
            block: create_test_block(1, "test"),
            sender: Uuid::new_v4(),
            signature,
            public_key,
        }
    }

    #[test]
    fn test_reject_zero_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        // Create message with zero signature
        let msg = PbftMessage::PrePrepare {
            view: 0,
            sequence: 1,
            block_hash: "hash".to_string(),
            block,
            sender,
            signature: Signature::from([0u8; 64]),
            public_key: keypair.verifying_key(),
        };

        assert!(!msg.verify_signature());
    }

    #[test]
    fn test_reject_random_signature() {
        let keypair = generate_signing_key().unwrap();
        let random_signature = Signature::from([42u8; 64]);
        let msg = create_block_with_signature(random_signature, keypair.verifying_key());

        assert!(!msg.verify_signature());
    }

    #[test]
    fn test_reject_wrong_public_key() {
        let keypair1 = generate_signing_key().unwrap();
        let keypair2 = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        // Sign with keypair1 but use keypair2's public key
        let message_bytes = format!("{}-{}-{}-{}", 0, 1, "hash", sender)
            .into_bytes();
        let signature = keypair1.sign(&message_bytes);

        let msg = PbftMessage::PrePrepare {
            view: 0,
            sequence: 1,
            block_hash: "hash".to_string(),
            block,
            sender,
            signature,
            public_key: keypair2.verifying_key(), // Wrong public key
        };

        assert!(!msg.verify_signature());
    }
}

#[cfg(test)]
mod public_key_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_public_key_from_pre_prepare() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let msg = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block,
            sender,
            &keypair
        );

        let extracted_key = msg.get_public_key();
        assert_eq!(*extracted_key, keypair.verifying_key());
    }

    #[test]
    fn test_extract_public_key_from_prepare() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_prepare(
            0,
            1,
            "hash".to_string(),
            sender,
            &keypair
        );

        let extracted_key = msg.get_public_key();
        assert_eq!(*extracted_key, keypair.verifying_key());
    }

    #[test]
    fn test_extract_public_key_from_commit() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_commit(
            0,
            1,
            "hash".to_string(),
            sender,
            &keypair
        );

        let extracted_key = msg.get_public_key();
        assert_eq!(*extracted_key, keypair.verifying_key());
    }

    #[test]
    fn test_extract_public_key_from_view_change() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_view_change(1, sender, &keypair);

        let extracted_key = msg.get_public_key();
        assert_eq!(*extracted_key, keypair.verifying_key());
    }
}

#[cfg(test)]
mod sender_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_sender_from_all_message_types() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let pre_prepare = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            create_test_block(1, "test"),
            sender,
            &keypair
        );

        let prepare = PbftMessage::create_prepare(
            0,
            1,
            "hash".to_string(),
            sender,
            &keypair
        );

        let commit = PbftMessage::create_commit(
            0,
            1,
            "hash".to_string(),
            sender,
            &keypair
        );

        let view_change = PbftMessage::create_view_change(1, sender, &keypair);

        assert_eq!(pre_prepare.get_sender(), sender);
        assert_eq!(prepare.get_sender(), sender);
        assert_eq!(commit.get_sender(), sender);
        assert_eq!(view_change.get_sender(), sender);
    }
}

#[cfg(test)]
mod signing_consistency_tests {
    use super::*;

    #[test]
    fn test_same_message_same_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let msg1 = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block.clone(),
            sender,
            &keypair
        );

        let msg2 = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block,
            sender,
            &keypair
        );

        // Same message should produce same signature
        let sig1 = msg1.get_signature();
        let sig2 = msg2.get_signature();
        assert_eq!(sig1.unwrap().to_bytes(), sig2.unwrap().to_bytes());
    }

    #[test]
    fn test_different_senders_different_signatures() {
        let keypair = generate_signing_key().unwrap();
        let sender1 = Uuid::new_v4();
        let sender2 = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let msg1 = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block.clone(),
            sender1,
            &keypair
        );

        let msg2 = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block,
            sender2,
            &keypair
        );

        // Different senders should produce different signatures
        let sig1 = msg1.get_signature();
        let sig2 = msg2.get_signature();
        assert_ne!(sig1.unwrap().to_bytes(), sig2.unwrap().to_bytes());
    }

    #[test]
    fn test_different_keypairs_different_signatures() {
        let keypair1 = generate_signing_key().unwrap();
        let keypair2 = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg1 = PbftMessage::create_prepare(
            0,
            1,
            "hash".to_string(),
            sender,
            &keypair1
        );

        let msg2 = PbftMessage::create_prepare(
            0,
            1,
            "hash".to_string(),
            sender,
            &keypair2
        );

        // Different keypairs should produce different signatures
        let sig1 = msg1.get_signature();
        let sig2 = msg2.get_signature();
        assert_ne!(sig1.unwrap().to_bytes(), sig2.unwrap().to_bytes());
    }
}
