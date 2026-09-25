//! Independent canonical vectors for Issue #10 grant and live-release records.

use ed25519_dalek::{Signature, SigningKey};
use provchain_org::network::convergence::PrivacyStateReceipt;
use provchain_org::privacy::{
    GrantDekEnvelope, LivePrivacyReleaseResponse, PrivacyControlTransition, PrivacyReleaseEnvelope,
    ProtectedPayloadCiphertext,
};
use sha2::{Digest, Sha256};

fn vectors() -> serde_json::Value {
    serde_json::from_str(include_str!("vectors/privacy_grant_v1.json"))
        .expect("checked-in Issue #10 vectors")
}

fn hex_field(vectors: &serde_json::Value, section: &str, field: &str) -> Vec<u8> {
    hex::decode(
        vectors[section][field]
            .as_str()
            .unwrap_or_else(|| panic!("missing vector field {section}.{field}")),
    )
    .unwrap_or_else(|error| panic!("invalid vector field {section}.{field}: {error}"))
}

fn array32(vectors: &serde_json::Value, section: &str, field: &str) -> [u8; 32] {
    hex_field(vectors, section, field)
        .try_into()
        .unwrap_or_else(|_| panic!("vector field {section}.{field} must be 32 bytes"))
}

fn parse_record(bytes: &[u8]) -> Vec<Vec<u8>> {
    assert_eq!(&bytes[..4], b"PCV1");
    let field_count = bytes[5] as usize;
    let mut offset = 6;
    let mut fields = Vec::with_capacity(field_count);
    for expected_tag in 1..=field_count {
        assert_eq!(bytes[offset], expected_tag as u8);
        offset += 1;
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("record field length"),
        ) as usize;
        offset += 4;
        fields.push(bytes[offset..offset + length].to_vec());
        offset += length;
    }
    assert_eq!(offset, bytes.len());
    fields
}

fn domain_hash(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize().into()
}

fn assert_signature(public_key: [u8; 32], digest: [u8; 32], signature: [u8; 64], valid: bool) {
    let verifying_key =
        ed25519_dalek::VerifyingKey::from_bytes(&public_key).expect("vector Ed25519 public key");
    let result = verifying_key.verify_strict(&digest, &Signature::from_bytes(&signature));
    assert_eq!(result.is_ok(), valid);
}

#[test]
fn independent_issue10_vectors_match_canonical_grant_and_release_surface() {
    let vectors = vectors();
    let grant_bytes = hex_field(&vectors, "grant_access", "complete_hex");
    let grant = PrivacyControlTransition::decode(&grant_bytes)
        .expect("decode independent GrantAccess vector");
    assert_eq!(grant.canonical_bytes(), grant_bytes);
    assert_eq!(
        grant.transition_id(),
        array32(&vectors, "grant_access", "grant_id_hex")
    );
    assert_eq!(
        grant.privacy_grant_id(),
        Some(array32(&vectors, "grant_access", "grant_id_hex"))
    );
    assert_eq!(
        GrantDekEnvelope::decode(&hex_field(&vectors, "grant_access", "envelope_hex"))
            .expect("decode independent grant envelope")
            .canonical_bytes(),
        hex_field(&vectors, "grant_access", "envelope_hex")
    );

    let revoke_bytes = hex_field(&vectors, "revoke_grant", "complete_hex");
    let revoke = PrivacyControlTransition::decode(&revoke_bytes)
        .expect("decode independent RevokeGrant vector");
    assert_eq!(revoke.canonical_bytes(), revoke_bytes);
    assert_eq!(
        revoke.transition_id(),
        array32(&vectors, "revoke_grant", "transition_id_hex")
    );
    assert_eq!(
        revoke.privacy_grant_id(),
        Some(array32(&vectors, "grant_access", "grant_id_hex"))
    );
    assert!(PrivacyControlTransition::decode(&hex_field(
        &vectors,
        "negative",
        "grant_truncated_complete_hex",
    ))
    .is_err());
    assert!(PrivacyControlTransition::decode(&hex_field(
        &vectors,
        "negative",
        "grant_owner_substitution_complete_hex",
    ))
    .is_err());
    assert!(PrivacyControlTransition::decode(&hex_field(
        &vectors,
        "negative",
        "grant_recipient_key_substitution_complete_hex",
    ))
    .is_err());

    let authorization_public =
        SigningKey::from_bytes(&array32(&vectors, "inputs", "authorization_seed_hex"))
            .verifying_key()
            .to_bytes();
    let grant_fields = parse_record(&grant_bytes);
    assert_signature(
        authorization_public,
        array32(&vectors, "grant_access", "authorization_digest_hex"),
        grant_fields[2].clone().try_into().expect("grant signature"),
        true,
    );
    let tampered_grant_fields = parse_record(&hex_field(
        &vectors,
        "negative",
        "grant_signature_tampered_complete_hex",
    ));
    assert_signature(
        authorization_public,
        array32(&vectors, "grant_access", "authorization_digest_hex"),
        tampered_grant_fields[2]
            .clone()
            .try_into()
            .expect("tampered grant signature"),
        false,
    );
    assert!(PrivacyControlTransition::decode(&hex_field(
        &vectors,
        "negative",
        "revoke_reused_or_wrong_grant_id_complete_hex",
    ))
    .is_err());

    let revoke_fields = parse_record(&revoke_bytes);
    assert_signature(
        authorization_public,
        array32(&vectors, "revoke_grant", "authorization_digest_hex"),
        revoke_fields[2]
            .clone()
            .try_into()
            .expect("revoke signature"),
        true,
    );
    let tampered_revoke_fields = parse_record(&hex_field(
        &vectors,
        "negative",
        "revoke_signature_tampered_complete_hex",
    ));
    assert_signature(
        authorization_public,
        array32(&vectors, "revoke_grant", "authorization_digest_hex"),
        tampered_revoke_fields[2]
            .clone()
            .try_into()
            .expect("tampered revoke signature"),
        false,
    );

    let payload_bytes = hex_field(&vectors, "live_release", "protected_payload_hex");
    assert_eq!(
        ProtectedPayloadCiphertext::decode(&payload_bytes)
            .expect("decode independent immutable payload")
            .canonical_bytes(),
        payload_bytes
    );
    for field in ["owner_response_hex", "grantee_response_hex"] {
        let response_bytes = hex_field(&vectors, "live_release", field);
        let response = LivePrivacyReleaseResponse::decode(&response_bytes)
            .expect("decode independent opaque live-release response");
        assert_eq!(response.canonical_bytes(), response_bytes);
        assert_eq!(response.payload().canonical_bytes(), payload_bytes);
        assert_eq!(
            response.encrypted_payload_commitment(),
            array32(&vectors, "live_release", "encrypted_payload_commitment_hex",)
        );
        match response.envelope() {
            PrivacyReleaseEnvelope::Owner(envelope) => assert_eq!(
                envelope.canonical_bytes(),
                hex_field(&vectors, "live_release", "owner_envelope_hex")
            ),
            PrivacyReleaseEnvelope::Grant(envelope) => assert_eq!(
                envelope.canonical_bytes(),
                hex_field(&vectors, "grant_access", "envelope_hex")
            ),
        }
    }
    assert!(LivePrivacyReleaseResponse::decode(&hex_field(
        &vectors,
        "negative",
        "live_response_commitment_tampered_hex",
    ))
    .is_err());

    let receipt_bytes = hex_field(&vectors, "privacy_state_receipt", "canonical_hex");
    let receipt = PrivacyStateReceipt::decode(&receipt_bytes)
        .expect("decode independent privacy-state receipt");
    assert_eq!(receipt.canonical_bytes(), receipt_bytes);
    assert_eq!(
        receipt.privacy_state_digest(),
        array32(
            &vectors,
            "privacy_state_receipt",
            "privacy_state_digest_hex",
        )
    );
    let node_key = SigningKey::from_bytes(&array32(&vectors, "inputs", "node_identity_seed_hex"));
    let receipt_signature_digest = domain_hash(
        b"provchain/privacy-state-receipt-signature/v1",
        &[
            &receipt.commit_receipt().canonical_bytes(),
            &receipt.privacy_state_digest(),
        ],
    );
    assert_signature(
        node_key.verifying_key().to_bytes(),
        receipt_signature_digest,
        receipt.node_signature(),
        true,
    );
    for field in [
        "privacy_state_digest_tampered_hex",
        "privacy_state_signature_tampered_hex",
    ] {
        let tampered = PrivacyStateReceipt::decode(&hex_field(&vectors, "negative", field))
            .expect("privacy receipt mutation remains structurally canonical");
        let tampered_digest = domain_hash(
            b"provchain/privacy-state-receipt-signature/v1",
            &[
                &tampered.commit_receipt().canonical_bytes(),
                &tampered.privacy_state_digest(),
            ],
        );
        assert_signature(
            node_key.verifying_key().to_bytes(),
            tampered_digest,
            tampered.node_signature(),
            false,
        );
    }

    let public_artifacts = [
        grant_bytes,
        revoke_bytes,
        hex_field(&vectors, "live_release", "owner_response_hex"),
        hex_field(&vectors, "live_release", "grantee_response_hex"),
        receipt_bytes,
    ];
    for artifact in public_artifacts {
        for secret_field in [
            "dek_fixture_hex",
            "authorization_seed_hex",
            "owner_wrapping_scalar_hex",
            "grantee_wrapping_scalar_hex",
            "hpke_seed_fixture_hex",
            "node_identity_seed_hex",
        ] {
            let secret = hex_field(&vectors, "inputs", secret_field);
            assert!(!artifact
                .windows(secret.len())
                .any(|window| window == secret));
        }
    }
}
