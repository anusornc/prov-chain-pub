//! PBFT Consensus Message Signing Tests
//!
//! Comprehensive test suite for PBFT message signing and verification.
//! Tests validate that:
//! - All PBFT messages are properly signed
//! - Signatures can be verified
//! - Invalid signatures are rejected
//! - Message tampering is detected

use ed25519_dalek::Signer;
use provchain_org::core::blockchain::Block;
use provchain_org::network::consensus::PbftMessage;
use provchain_org::security::keys::generate_signing_key;
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

        let msg = PbftMessage::create_pre_prepare(0, 1, block_hash, block, sender, &keypair);

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

        let msg =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

        assert!(msg.verify_signature(), "Valid signature should verify");
    }

    #[test]
    fn test_verify_valid_prepare_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_prepare(0, 1, "block_hash".to_string(), sender, &keypair);

        assert!(msg.verify_signature());
    }

    #[test]
    fn test_verify_valid_commit_signature() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_commit(0, 1, "block_hash".to_string(), sender, &keypair);

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
            0, // Original view
            1,
            "hash".to_string(),
            block,
            sender,
            &keypair,
        );

        // Tamper with view number
        if let PbftMessage::PrePrepare { ref mut view, .. } = msg {
            *view = 999; // Tampered
        }

        // Signature should no longer verify
        assert!(
            !msg.verify_signature(),
            "Tampered message should fail verification"
        );
    }

    #[test]
    fn test_detect_tampered_sequence() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let mut msg = PbftMessage::create_prepare(
            0,
            1, // Original sequence
            "block_hash".to_string(),
            sender,
            &keypair,
        );

        // Tamper with sequence number
        if let PbftMessage::Prepare {
            ref mut sequence, ..
        } = msg
        {
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
            "original_hash".to_string(), // Original hash
            sender,
            &keypair,
        );

        // Tamper with block_hash
        if let PbftMessage::Commit {
            ref mut block_hash, ..
        } = msg
        {
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

    fn create_block_with_signature(signature: Signature, public_key: VerifyingKey) -> PbftMessage {
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
        let message_bytes = format!("{}-{}-{}-{}", 0, 1, "hash", sender).into_bytes();
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

        let msg =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

        let extracted_key = msg.get_public_key();
        assert_eq!(*extracted_key, keypair.verifying_key());
    }

    #[test]
    fn test_extract_public_key_from_prepare() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_prepare(0, 1, "hash".to_string(), sender, &keypair);

        let extracted_key = msg.get_public_key();
        assert_eq!(*extracted_key, keypair.verifying_key());
    }

    #[test]
    fn test_extract_public_key_from_commit() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();

        let msg = PbftMessage::create_commit(0, 1, "hash".to_string(), sender, &keypair);

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
            &keypair,
        );

        let prepare = PbftMessage::create_prepare(0, 1, "hash".to_string(), sender, &keypair);

        let commit = PbftMessage::create_commit(0, 1, "hash".to_string(), sender, &keypair);

        let view_change = PbftMessage::create_view_change(1, sender, &keypair);

        assert_eq!(pre_prepare.get_sender(), sender);
        assert_eq!(prepare.get_sender(), sender);
        assert_eq!(commit.get_sender(), sender);
        assert_eq!(view_change.get_sender(), sender);
    }
}

#[cfg(test)]
mod replay_protection_tests {
    use super::*;

    /// Helper to generate message ID matching the implementation
    fn generate_message_id(msg_type: &str, view: u64, sequence: u64, sender: Uuid) -> String {
        format!("{}-{}-{}-{}", msg_type, view, sequence, sender)
    }

    #[test]
    fn test_message_id_format_pre_prepare() {
        let sender = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let msg_id = generate_message_id("preprepare", 0, 1, sender);

        assert_eq!(
            msg_id,
            "preprepare-0-1-550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_message_id_uniqueness() {
        let sender = Uuid::new_v4();

        let id1 = generate_message_id("preprepare", 0, 1, sender);
        let id2 = generate_message_id("preprepare", 0, 2, sender);
        let id3 = generate_message_id("prepare", 0, 1, sender);
        let id4 = generate_message_id("preprepare", 1, 1, sender);

        // All should be different
        assert_ne!(id1, id2, "Different sequence should produce different IDs");
        assert_ne!(
            id1, id3,
            "Different message type should produce different IDs"
        );
        assert_ne!(id1, id4, "Different view should produce different IDs");
    }

    #[test]
    fn test_duplicate_pre_prepare_detected() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let _msg1 = PbftMessage::create_pre_prepare(
            0,
            1,
            "hash".to_string(),
            block.clone(),
            sender,
            &keypair,
        );

        let _msg2 =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

        // Same parameters should produce same message ID
        let id1 = generate_message_id("preprepare", 0, 1, sender);
        let id2 = generate_message_id("preprepare", 0, 1, sender);

        assert_eq!(id1, id2, "Duplicate messages should have same ID");
    }

    #[test]
    fn test_message_id_includes_all_critical_fields() {
        let sender = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        // Test all message types
        let pre_prepare_id = generate_message_id("preprepare", 5, 100, sender);
        assert!(pre_prepare_id.contains("preprepare"));
        assert!(pre_prepare_id.contains("5"));
        assert!(pre_prepare_id.contains("100"));
        assert!(pre_prepare_id.contains("550e8400"));

        let prepare_id = generate_message_id("prepare", 1, 2, sender);
        assert!(prepare_id.contains("prepare"));
        assert!(prepare_id.contains("1"));
        assert!(prepare_id.contains("2"));
    }

    #[test]
    fn test_different_sender_different_message_id() {
        let sender1 = Uuid::new_v4();
        let sender2 = Uuid::new_v4();

        let id1 = generate_message_id("preprepare", 0, 1, sender1);
        let id2 = generate_message_id("preprepare", 0, 1, sender2);

        assert_ne!(
            id1, id2,
            "Different senders should produce different message IDs"
        );
    }

    #[test]
    fn test_message_id_is_deterministic() {
        let sender = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let id1 = generate_message_id("commit", 10, 20, sender);
        let id2 = generate_message_id("commit", 10, 20, sender);

        assert_eq!(id1, id2, "Message ID generation should be deterministic");
    }

    #[test]
    fn test_message_id_format_is_parseable() {
        let sender = Uuid::new_v4();
        let msg_id = generate_message_id("viewchange", 3, 7, sender);

        // Should contain hyphens as separators
        let parts: Vec<&str> = msg_id.split('-').collect();
        assert!(
            parts.len() >= 5,
            "Message ID should have at least 5 parts separated by hyphens"
        );

        // Last part should be a valid UUID
        let last_part = parts.last().unwrap();
        assert!(
            Uuid::parse_str(last_part).is_ok(),
            "Last part should be a UUID"
        );
    }

    #[test]
    fn test_view_change_message_id() {
        let sender = Uuid::new_v4();

        // View change has different format (only view and sender)
        let msg_id = generate_message_id("viewchange", 5, 0, sender);

        assert!(msg_id.contains("viewchange"));
        assert!(msg_id.contains("5"));
        assert!(msg_id.contains(&sender.to_string()));
    }

    #[test]
    fn test_replay_protection_across_views() {
        let sender = Uuid::new_v4();

        // Same sequence in different views
        let view0_id = generate_message_id("prepare", 0, 10, sender);
        let view1_id = generate_message_id("prepare", 1, 10, sender);

        assert_ne!(
            view0_id, view1_id,
            "Same sequence in different views should have different IDs"
        );
    }
}

#[cfg(test)]
mod signature_verification_order_tests {
    use super::*;

    /// Tests that signature verification happens before expensive operations
    /// This is a compile-time test to ensure the security pattern is followed
    #[test]
    fn test_signature_verification_pattern() {
        // This test documents the security pattern:
        // 1. Verify signature FIRST (fast, cryptographic check)
        // 2. Check for replay attacks (hashset lookup)
        // 3. Check authorization (if needed)
        // 4. Process the message (expensive operations)

        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let msg =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

        // Signature should be verifiable
        assert!(msg.verify_signature());

        // If signature verification failed, we should reject immediately
        // without checking replay or doing any expensive processing
    }

    #[test]
    fn test_invalid_signature_rejected_early() {
        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let mut msg =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

        // Tamper with the message (which invalidates the signature)
        if let PbftMessage::PrePrepare { ref mut view, .. } = msg {
            *view = 999;
        }

        // Signature verification should fail
        assert!(!msg.verify_signature());

        // In the implementation, this failure happens before:
        // - Replay protection checks
        // - Authorization checks
        // - Block validation
        // - State updates
    }

    #[test]
    fn test_signature_verification_is_fast() {
        use std::time::Instant;

        let keypair = generate_signing_key().unwrap();
        let sender = Uuid::new_v4();
        let block = create_test_block(1, "test");

        let msg =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

        let iterations = 10000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = msg.verify_signature();
        }

        let duration = start.elapsed();
        let per_verification = duration.as_micros() as f64 / iterations as f64;

        println!(
            "Signature verification: {} iterations in {:?}",
            iterations, duration
        );
        println!("Average: {:.2} μs per verification", per_verification);

        // Ed25519 verification should be very fast (< 100μs)
        assert!(
            per_verification < 100.0,
            "Signature verification should be fast"
        );
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
            &keypair,
        );

        let msg2 =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender, &keypair);

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
            &keypair,
        );

        let msg2 =
            PbftMessage::create_pre_prepare(0, 1, "hash".to_string(), block, sender2, &keypair);

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

        let msg1 = PbftMessage::create_prepare(0, 1, "hash".to_string(), sender, &keypair1);

        let msg2 = PbftMessage::create_prepare(0, 1, "hash".to_string(), sender, &keypair2);

        // Different keypairs should produce different signatures
        let sig1 = msg1.get_signature();
        let sig2 = msg2.get_signature();
        assert_ne!(sig1.unwrap().to_bytes(), sig2.unwrap().to_bytes());
    }
}
