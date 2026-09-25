#!/usr/bin/env python3
"""Generate independent Issue #10 grant and live-release vectors.

The reference implementation uses only the Python standard library and the
independent curve/AEAD primitives already used by the predecessor vector
generators.  It never imports ProvChain or any Rust implementation.  Secrets
are fixed, labelled test fixtures and are included only to reproduce opaque
public test records.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from generate_privacy_lifecycle_vectors import (
    dhash,
    ed25519_public,
    ed25519_sign,
    fingerprint,
    participant_reference,
    p256_public,
    record,
)
from generate_protected_object_vectors import build_vector as build_protected_vector
from generate_protected_object_vectors import hpke_seal


SUITE = b"ProtectedDataSuiteV1"
NETWORK_ID = "provchain.issue9.vectors"
PROFILE_ID = "privacy.protected.v1"
PROFILE_HASH = bytes([0xA6]) * 32
OWNER = bytes.fromhex("00112233445566778899aabbccddeeff")
GRANTEE = bytes.fromhex("11223344556647788899aabbccddeeff")
OBJECT_ID = bytes.fromhex("91000000000040018000000000000009")
AUTHORIZATION_SEED = bytes([0x22]) * 32
OWNER_WRAPPING_SCALAR = bytes([0x33]) * 32
GRANTEE_WRAPPING_SCALAR = bytes([0x66]) * 32
DEK = bytes([0x44]) * 32
HPKE_SEED = bytes([0x55]) * 32
NODE_SEED = bytes([0x99]) * 32
PERMISSION = b"\x01"
GRANT_VARIANT = 0x05
REVOKE_VARIANT = 0x06


def common_fields(variant: int, anchor: list[bytes]) -> list[bytes]:
    return [
        b"PrivacyControlV1",
        b"CanonicalPrivacyEncodingV1",
        bytes((variant,)),
        *anchor,
    ]


def anchor_fields(position: int, revision: int, parent_prefix: bytes, parent_envelope: bytes) -> list[bytes]:
    return [
        NETWORK_ID.encode("ascii"),
        PROFILE_ID.encode("ascii"),
        PROFILE_HASH,
        position.to_bytes(8, "big"),
        revision.to_bytes(8, "big"),
        parent_prefix,
        b"\x01" + parent_envelope,
    ]


def length_prefixed_hash(domain: bytes, *parts: bytes) -> bytes:
    digest = hashlib.sha256()
    digest.update(domain)
    for part in parts:
        digest.update(len(part).to_bytes(8, "big"))
        digest.update(part)
    return digest.digest()


def write_text(value: str) -> bytes:
    encoded = value.encode("ascii")
    return len(encoded).to_bytes(4, "big") + encoded


def build_vectors() -> dict[str, Any]:
    protected = build_protected_vector()
    object_inputs = protected["inputs"]
    object_values = protected["create_protected_object"]
    payload = bytes.fromhex(object_values["payload_hex"])
    object_pcc = bytes.fromhex(object_values["pcc_hex"])
    object_epc = bytes.fromhex(object_values["epc_hex"])
    owner_envelope = bytes.fromhex(object_values["owner_envelope_hex"])
    owner_authorization_reference = bytes.fromhex(object_values["authorization_reference_hex"])
    owner_wrapping_reference = bytes.fromhex(object_values["wrapping_reference_hex"])
    grantee_public = p256_public(GRANTEE_WRAPPING_SCALAR)
    grantee_fingerprint = fingerprint(
        "ProtectedDataSuiteV1", 0x02, 0x02, grantee_public
    )
    grantee_wrapping_reference = participant_reference(
        GRANTEE, 0x02, 1, 0x02, 0x02, grantee_fingerprint
    )

    grant_anchor = anchor_fields(3, 4, bytes([0xD3]) * 32, bytes([0xE3]) * 32)
    grant_header = record(
        0x25,
        [
            SUITE,
            *grant_anchor,
            OBJECT_ID,
            OWNER,
            GRANTEE,
            PERMISSION,
            grantee_wrapping_reference,
            object_pcc,
            object_epc,
        ],
    )
    grant_delivery_context_input = record(
        0x24,
        common_fields(GRANT_VARIANT, grant_anchor)
        + [
            OBJECT_ID,
            OWNER,
            GRANTEE,
            PERMISSION,
            SUITE,
            object_pcc,
            object_epc,
            grantee_wrapping_reference,
            b"\x26",
            SUITE,
        ],
    )
    grant_delivery_context = dhash(
        "provchain/privacy-grant/delivery-context/v1", grant_delivery_context_input
    )
    grant_header_hash = dhash(
        "provchain/protected-data/hpke-grant-header/v1", grant_header
    )
    grant_info = grant_delivery_context + grant_header_hash
    hpke_ikm, grant_encapsulated, grant_wrapped = hpke_seal(
        grantee_public, grant_info, DEK
    )
    grant_envelope = record(
        0x26,
        [SUITE, grant_delivery_context, grant_header, grant_encapsulated, grant_wrapped],
    )
    grant_core = record(
        0x34,
        common_fields(GRANT_VARIANT, grant_anchor)
        + [
            OBJECT_ID,
            OWNER,
            GRANTEE,
            PERMISSION,
            grant_envelope,
            b"\x02",
            owner_authorization_reference,
        ],
    )
    grant_id = dhash("provchain/privacy-grant/grant-id/v1", grant_core)
    grant_authorization_digest = dhash(
        "provchain/privacy-grant/owner-authorization/v1", grant_id + grant_core
    )
    grant_authorization_signature = ed25519_sign(
        AUTHORIZATION_SEED, grant_authorization_digest
    )
    grant_complete = record(
        0x44, [grant_core, grant_id, grant_authorization_signature]
    )

    revoke_anchor = anchor_fields(4, 5, bytes([0xD4]) * 32, bytes([0xE4]) * 32)
    revoke_core = record(
        0x35,
        common_fields(REVOKE_VARIANT, revoke_anchor)
        + [
            grant_id,
            OBJECT_ID,
            GRANTEE,
            OWNER,
            PERMISSION,
            (3).to_bytes(8, "big"),
            b"\x02",
            owner_authorization_reference,
        ],
    )
    revoke_transition_id = dhash(
        "provchain/privacy-grant/revoke-transition-id/v1", revoke_core
    )
    revoke_authorization_digest = dhash(
        "provchain/privacy-grant/revoke-owner-authorization/v1",
        revoke_transition_id + revoke_core,
    )
    revoke_authorization_signature = ed25519_sign(
        AUTHORIZATION_SEED, revoke_authorization_digest
    )
    revoke_complete = record(
        0x45, [revoke_core, revoke_transition_id, revoke_authorization_signature]
    )

    owner_evidence = record(
        0x50,
        [
            NETWORK_ID.encode("ascii"),
            PROFILE_ID.encode("ascii"),
            PROFILE_HASH,
            (2).to_bytes(8, "big"),
            bytes([0xB2]) * 32,
            OBJECT_ID,
            OWNER,
            owner_wrapping_reference,
            b"\x01",
        ],
    )
    owner_response = record(
        0x52, [owner_evidence, payload, owner_envelope, object_epc]
    )
    grantee_evidence = record(
        0x51,
        [
            NETWORK_ID.encode("ascii"),
            PROFILE_ID.encode("ascii"),
            PROFILE_HASH,
            (3).to_bytes(8, "big"),
            bytes([0xF3]) * 32,
            OBJECT_ID,
            GRANTEE,
            grantee_wrapping_reference,
            b"\x02",
            grant_id,
        ],
    )
    grantee_response = record(
        0x52, [grantee_evidence, payload, grant_envelope, object_epc]
    )

    node_id = bytes.fromhex("00000000000040008000000000000010")
    manifest_digest = bytes([0xAB]) * 32
    receipt_envelope_hash = bytes([0xCD]) * 32
    receipt_prefix_hash = bytes([0xEF]) * 32
    commit_unsigned = (
        b"PROVCHAIN_COMMIT_RECEIPT_V1"
        + (1).to_bytes(2, "big")
        + node_id
        + write_text(NETWORK_ID)
        + write_text(PROFILE_ID)
        + write_text("issue10.membership")
        + (1).to_bytes(8, "big")
        + manifest_digest
        + (3).to_bytes(8, "big")
        + receipt_envelope_hash
        + receipt_prefix_hash
    )
    commit_signature_digest = length_prefixed_hash(
        b"provchain/commit-receipt-signature/v1", commit_unsigned
    )
    commit_receipt = commit_unsigned + ed25519_sign(NODE_SEED, commit_signature_digest)
    privacy_state_digest = bytes([0xA9]) * 32
    privacy_receipt_signature_digest = length_prefixed_hash(
        b"provchain/privacy-state-receipt-signature/v1",
        commit_receipt,
        privacy_state_digest,
    )
    privacy_state_receipt = (
        b"PROVCHAIN_PRIVACY_STATE_RECEIPT_V1"
        + (1).to_bytes(2, "big")
        + len(commit_receipt).to_bytes(4, "big")
        + commit_receipt
        + privacy_state_digest
        + ed25519_sign(NODE_SEED, privacy_receipt_signature_digest)
    )

    tampered_grant_signature = bytearray(grant_complete)
    tampered_grant_signature[-1] ^= 1
    tampered_grant_owner = bytearray(grant_complete)
    owner_offset = grant_complete.find(OWNER)
    assert owner_offset >= 0
    tampered_grant_owner[owner_offset] ^= 1
    tampered_grantee_reference = bytearray(grant_complete)
    reference_offset = grant_complete.find(grantee_wrapping_reference)
    assert reference_offset >= 0
    tampered_grantee_reference[reference_offset + len(grantee_wrapping_reference) - 1] ^= 1
    tampered_revoke_signature = bytearray(revoke_complete)
    tampered_revoke_signature[-1] ^= 1
    tampered_revoke_grant_id = bytearray(revoke_complete)
    grant_id_offset = revoke_complete.find(grant_id)
    assert grant_id_offset >= 0
    tampered_revoke_grant_id[grant_id_offset] ^= 1
    tampered_response = bytearray(grantee_response)
    tampered_response[-1] ^= 1
    tampered_privacy_receipt_digest = bytearray(privacy_state_receipt)
    privacy_digest_offset = privacy_state_receipt.find(privacy_state_digest)
    assert privacy_digest_offset >= 0
    tampered_privacy_receipt_digest[privacy_digest_offset] ^= 1
    tampered_privacy_receipt_signature = bytearray(privacy_state_receipt)
    tampered_privacy_receipt_signature[-1] ^= 1

    source_path = Path(__file__)
    lock_path = source_path.parent.parent / "Cargo.lock"
    return {
        "manifest": {
            "schema": "ProvChainPrivacyGrantVectorsV1",
            "scope": "Issue #10 GrantAccess, RevokeGrant, privacy-state receipt, and opaque live-release response",
            "canonical_encoding": "CanonicalPrivacyEncodingV1",
            "generator": {
                "implementation": "independent Python standard-library reference",
                "source": "scripts/generate_privacy_grant_vectors.py",
                "source_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
                "generate_command": "python3 scripts/generate_privacy_grant_vectors.py --output tests/vectors/privacy_grant_v1.json",
                "verify_command": "python3 scripts/generate_privacy_grant_vectors.py --verify tests/vectors/privacy_grant_v1.json",
            },
            "rust_msrv": "1.87",
            "cargo_lock_sha256": hashlib.sha256(lock_path.read_bytes()).hexdigest(),
            "protected_data_suite": "ProtectedDataSuiteV1",
        },
        "inputs": {
            "network_id": NETWORK_ID,
            "profile_id": PROFILE_ID,
            "profile_content_hash_hex": PROFILE_HASH.hex(),
            "owner_uuid": "00112233-4455-6677-8899-aabbccddeeff",
            "grantee_uuid": "11223344-5566-4778-8899-aabbccddeeff",
            "object_uuid": object_inputs["object_uuid"],
            "authorization_seed_hex": AUTHORIZATION_SEED.hex(),
            "owner_wrapping_scalar_hex": OWNER_WRAPPING_SCALAR.hex(),
            "grantee_wrapping_scalar_hex": GRANTEE_WRAPPING_SCALAR.hex(),
            "dek_fixture_hex": DEK.hex(),
            "hpke_seed_fixture_hex": HPKE_SEED.hex(),
            "node_identity_seed_hex": NODE_SEED.hex(),
            "node_id_hex": node_id.hex(),
        },
        "grant_access": {
            "grantee_public_key_hex": grantee_public.hex(),
            "grantee_wrapping_reference_hex": grantee_wrapping_reference.hex(),
            "anchor_parent_prefix_hash_hex": (bytes([0xD3]) * 32).hex(),
            "anchor_parent_envelope_hash_hex": (bytes([0xE3]) * 32).hex(),
            "delivery_context_input_hex": grant_delivery_context_input.hex(),
            "delivery_context_hex": grant_delivery_context.hex(),
            "header_hex": grant_header.hex(),
            "header_hash_hex": grant_header_hash.hex(),
            "hpke_info_hex": grant_info.hex(),
            "hpke_ikm_hex": hpke_ikm.hex(),
            "encapsulated_key_hex": grant_encapsulated.hex(),
            "wrapped_dek_hex": grant_wrapped.hex(),
            "envelope_hex": grant_envelope.hex(),
            "core_hex": grant_core.hex(),
            "grant_id_hex": grant_id.hex(),
            "authorization_digest_hex": grant_authorization_digest.hex(),
            "authorization_signature_hex": grant_authorization_signature.hex(),
            "complete_hex": grant_complete.hex(),
        },
        "revoke_grant": {
            "creation_ledger_position": 3,
            "core_hex": revoke_core.hex(),
            "transition_id_hex": revoke_transition_id.hex(),
            "authorization_digest_hex": revoke_authorization_digest.hex(),
            "authorization_signature_hex": revoke_authorization_signature.hex(),
            "complete_hex": revoke_complete.hex(),
        },
        "privacy_state_receipt": {
            "manifest_id": "issue10.membership",
            "manifest_version": 1,
            "manifest_digest_hex": manifest_digest.hex(),
            "ledger_position": 3,
            "envelope_hash_hex": receipt_envelope_hash.hex(),
            "ledger_prefix_hash_hex": receipt_prefix_hash.hex(),
            "privacy_state_digest_hex": privacy_state_digest.hex(),
            "commit_receipt_unsigned_hex": commit_unsigned.hex(),
            "commit_receipt_hex": commit_receipt.hex(),
            "canonical_hex": privacy_state_receipt.hex(),
        },
        "live_release": {
            "protected_payload_hex": payload.hex(),
            "encrypted_payload_commitment_hex": object_epc.hex(),
            "owner_envelope_hex": owner_envelope.hex(),
            "owner_evidence_hex": owner_evidence.hex(),
            "owner_response_hex": owner_response.hex(),
            "grantee_evidence_hex": grantee_evidence.hex(),
            "grantee_response_hex": grantee_response.hex(),
        },
        "negative": {
            "grant_signature_tampered_complete_hex": bytes(tampered_grant_signature).hex(),
            "grant_owner_substitution_complete_hex": bytes(tampered_grant_owner).hex(),
            "grant_recipient_key_substitution_complete_hex": bytes(
                tampered_grantee_reference
            ).hex(),
            "grant_truncated_complete_hex": grant_complete[:-1].hex(),
            "revoke_signature_tampered_complete_hex": bytes(tampered_revoke_signature).hex(),
            "revoke_reused_or_wrong_grant_id_complete_hex": bytes(
                tampered_revoke_grant_id
            ).hex(),
            "live_response_commitment_tampered_hex": bytes(tampered_response).hex(),
            "privacy_state_digest_tampered_hex": bytes(
                tampered_privacy_receipt_digest
            ).hex(),
            "privacy_state_signature_tampered_hex": bytes(
                tampered_privacy_receipt_signature
            ).hex(),
            "missing_privacy_receipts": 2,
            "degraded_reference_nodes": 1,
            "partitioned_reference_nodes": 1,
            "race_before_revoke_linearization": "release_allowed_at_pre_revoke_prefix",
            "race_after_revoke_linearization": "release_denied_at_post_revoke_prefix",
        },
    }


def encoded(vector: dict[str, Any]) -> str:
    return json.dumps(vector, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    generated = encoded(build_vectors())
    if args.verify is not None:
        expected = args.verify.read_text(encoding="utf-8")
        if expected != generated:
            raise SystemExit(f"privacy grant vector mismatch: {args.verify}")
        print(f"verified {args.verify}")
        return 0
    if args.output is not None:
        args.output.write_text(generated, encoding="utf-8")
        print(f"wrote {args.output}")
        return 0
    print(generated, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
