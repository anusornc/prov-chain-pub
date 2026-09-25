#!/usr/bin/env python3
"""Generate Issue #11 bridge bytes without calling ProvChain Rust code."""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
import subprocess
import tempfile
import uuid
from pathlib import Path


# These state commitments are fixed upstream RDF-canonicalization fixtures. The
# independent generator rebuilds every bridge-dependent envelope byte itself.
EMPTY_STATE_COMMITMENT = bytes.fromhex(
    "47177c7bb2c11ccdb628711cb4c0ee00c109d6f628a59eb0c151b00a2b628bc9"
)
PAYLOAD_POST_STATE_COMMITMENT = bytes.fromhex(
    "d3a7c6f71cf4fb482d42e94a60bb45d367599e80358d8e9f0a2a1ce9e8f4ef9b"
)


def u16(value: int) -> bytes:
    return struct.pack(">H", value)


def u32(value: int) -> bytes:
    return struct.pack(">I", value)


def u64(value: int) -> bytes:
    return struct.pack(">Q", value)


def text32(value: str) -> bytes:
    encoded = value.encode("ascii")
    return u32(len(encoded)) + encoded


def text64(value: str) -> bytes:
    encoded = value.encode("ascii")
    return u64(len(encoded)) + encoded


def bridge_record(tag: int, fields: list[bytes]) -> bytes:
    encoded = b"BCV1" + bytes([tag, len(fields)])
    for field_tag, value in enumerate(fields, start=1):
        encoded += bytes([field_tag]) + u32(len(value)) + value
    return encoded


def bridge_hash(label: bytes, record: bytes) -> bytes:
    return hashlib.sha256(u16(len(label)) + label + u32(len(record)) + record).digest()


def contract_hash(domain: bytes, parts: list[bytes]) -> bytes:
    data = domain
    for part in parts:
        data += u64(len(part)) + part
    return hashlib.sha256(data).digest()


def private_der(seed: int) -> bytes:
    return bytes.fromhex("302e020100300506032b657004220420") + bytes([seed]) * 32


def openssl_ed25519(seed: int, message: bytes) -> bytes:
    with tempfile.TemporaryDirectory(prefix="provchain-bridge-vector-") as directory:
        key_path = Path(directory) / "key.der"
        pem_path = Path(directory) / "key.pem"
        message_path = Path(directory) / "message.bin"
        key_path.write_bytes(private_der(seed))
        message_path.write_bytes(message)
        subprocess.run(
            [
                "openssl",
                "pkey",
                "-inform",
                "DER",
                "-in",
                str(key_path),
                "-out",
                str(pem_path),
            ],
            check=True,
            capture_output=True,
        )
        completed = subprocess.run(
            [
                "openssl",
                "pkeyutl",
                "-sign",
                "-rawin",
                "-inkey",
                str(pem_path),
                "-in",
                str(message_path),
            ],
            check=True,
            capture_output=True,
        )
        if len(completed.stdout) != 64:
            raise ValueError("OpenSSL returned a non-Ed25519 signature")
        return completed.stdout


def openssl_public_key(seed: int) -> bytes:
    with tempfile.TemporaryDirectory(prefix="provchain-bridge-vector-") as directory:
        key_path = Path(directory) / "key.der"
        key_path.write_bytes(private_der(seed))
        completed = subprocess.run(
            [
                "openssl",
                "pkey",
                "-inform",
                "DER",
                "-in",
                str(key_path),
                "-pubout",
                "-outform",
                "DER",
            ],
            check=True,
            capture_output=True,
        )
        return completed.stdout[-32:]


def membership_manifest() -> tuple[bytes, bytes, bytes]:
    network_id = "provchain.issue11.source"
    profile_id = "issue11.source.reference"
    manifest_id = "issue11.source.membership"
    members = []
    for node_number, identity_seed, validator_seed in [
        (110, 12, 15),
        (120, 13, 16),
        (130, 14, 17),
    ]:
        node = uuid.UUID(int=node_number)
        members.append(
            node.bytes
            + openssl_public_key(identity_seed)
            + bytes([1])
            + u64(2)
            + bytes([1, 2])
            + bytes([1])
            + openssl_public_key(validator_seed)
        )
    canonical = (
        b"PROVCHAIN_MEMBERSHIP_MANIFEST_V1"
        + u16(1)
        + text64(manifest_id)
        + u64(11)
        + text64(network_id)
        + text64(profile_id)
        + u64(len(members))
        + b"".join(members)
    )
    digest = contract_hash(b"provchain/membership-manifest-digest/v1", [canonical])
    signature_message = contract_hash(
        b"provchain/membership-manifest-signature/v1", [digest]
    )
    signature = openssl_ed25519(11, signature_message)
    signed = (
        b"PROVCHAIN_SIGNED_MEMBERSHIP_MANIFEST_V1"
        + u32(len(canonical))
        + canonical
        + signature
    )
    return canonical, digest, signed


def network_profile(manifest_digest: bytes) -> bytes:
    authority_keys = [openssl_public_key(seed).hex() for seed in [15, 16, 17]]
    return (
        b"PROVCHAIN_NETWORK_PROFILE_CONTENT_V1"
        + text32("issue11.source.reference")
        + text32("provchain.issue11.source")
        + text32("poa")
        + u32(len(authority_keys))
        + b"".join(text32(key) for key in authority_keys)
        + u64(10)
        + u64(1_048_576)
        + text32("provchain.test.semantic")
        + text32("1.0.0")
        + text32("c24c2ef943de01d15fce5a22b798c389e20075055d07eda41c7820699bafa69c")
        + text32("provchain.semantic-execution.v1")
        + text32("strict")
        + bytes([1])
        + text32("issue11.source.membership")
        + u64(11)
        + text32(manifest_digest.hex())
        + bytes([0])
        + bytes([1])
        + bytes([0x31]) * 32
        + text32("provchain.issue11.target")
        + bytes([0x42]) * 32
        + text32("ProvChainBridgeSuiteV1")
        + text32("ExactPublicPayloadV1")
        + text32("Rdfc10Sha256NQuadsV1")
        + text32("OrdinaryProvenanceV1")
        + b"".join(
            u64(bound)
            for bound in [
                262_144,
                1_024,
                32_768,
                65_536,
                393_216,
                4_096,
                3,
                503_883,
                503_968,
                262_187,
                1_048_576,
            ]
        )
    )


def generate() -> dict[str, object]:
    _, manifest_digest, signed_manifest = membership_manifest()
    network_profile_bytes = network_profile(manifest_digest)
    # The independent implementation deliberately duplicates the closed codecs;
    # sharing Rust helpers would turn this into a self-confirming golden vector.
    source_profile = network_profile_bytes
    source_profile_hash = hashlib.sha256(source_profile).digest()
    payload = b'<urn:issue11:batch:2> <urn:issue11:status> "exact-public-copy" .'
    export_core = bridge_record(
        0x01,
        [
            b"provchain.issue11.source",
            bytes([0x31]) * 32,
            b"issue11.source.reference",
            source_profile_hash,
            u64(0),
            b"provchain.issue11.target",
            bytes([0x42]) * 32,
            bytes([1]),
            bytes([1]),
            hashlib.sha256(payload).digest(),
        ],
    )
    transfer_id = bridge_hash(b"provchain/bridge/transfer-id/v1", export_core)
    declaration = bridge_record(0x02, [export_core, transfer_id])
    proposer_public_key = openssl_public_key(15)
    envelope_fields = (
        u16(4)
        + bytes([1])
        + text32("provchain.issue11.source")
        + text32("issue11.source.reference")
        + text32("provchain.test.semantic")
        + text32("1.0.0")
        + text32("c24c2ef943de01d15fce5a22b798c389e20075055d07eda41c7820699bafa69c")
        + text32("provchain.semantic-execution.v1")
        + bytes([1])
        + u64(0)
        + bytes(32)
        + EMPTY_STATE_COMMITMENT
        + u64(21_000)
        + u32(len(payload))
        + payload
        + bytes([0])
        + u32(len(declaration))
        + declaration
        + PAYLOAD_POST_STATE_COMMITMENT
        + proposer_public_key
    )
    proposal_body = b"PROVCHAIN_PROPOSAL_V2" + envelope_fields
    proposal_digest = contract_hash(
        b"provchain/ordinary-proposal-digest/v1", [proposal_body]
    )
    proposer_signature = openssl_ed25519(15, proposal_digest)
    source_envelope_without_hash = (
        b"PROVCHAIN_ADMITTED_ENVELOPE_V2"
        + envelope_fields
        + proposal_digest
        + proposer_signature
    )
    envelope_hash = contract_hash(
        b"provchain/admitted-envelope-hash/v1", [source_envelope_without_hash]
    )
    source_envelope = source_envelope_without_hash + envelope_hash
    genesis_prefix = contract_hash(b"provchain/ledger-prefix-genesis/v1", [])
    ledger_prefix_hash = contract_hash(
        b"provchain/ledger-prefix-step/v1",
        [genesis_prefix, u64(0), source_envelope],
    )
    receipts = []
    for node_number, identity_seed in [(110, 12), (120, 13), (130, 14)]:
        receipt_core = bridge_record(
            0x03,
            [
                b"provchain.issue11.source",
                bytes([0x31]) * 32,
                b"issue11.source.reference",
                source_profile_hash,
                b"issue11.source.membership",
                u64(11),
                manifest_digest,
                str(uuid.UUID(int=node_number)).encode("ascii"),
                u64(0),
                envelope_hash,
                ledger_prefix_hash,
            ],
        )
        message = bridge_hash(
            b"provchain/bridge/commit-receipt-signature/v1", receipt_core
        )
        receipts.append(
            bridge_record(0x04, [receipt_core, bytes([1]), openssl_ed25519(identity_seed, message)])
        )
    receipt_sequence = b"".join(u32(len(receipt)) + receipt for receipt in receipts)
    proof = bridge_record(
        0x10,
        [transfer_id, source_profile, signed_manifest, source_envelope, receipt_sequence],
    )
    return {
        "format": "ProvChainBridgeSuiteV1 independent OpenSSL vector",
        "expected": {
            "source_profile": source_profile.hex(),
            "signed_manifest": signed_manifest.hex(),
            "source_envelope": source_envelope.hex(),
            "declaration": declaration.hex(),
            "transfer_id": transfer_id.hex(),
            "envelope_hash": envelope_hash.hex(),
            "ledger_prefix_hash": ledger_prefix_hash.hex(),
            "reproducibility": {
                "source_position": 0,
                "declaration_sha256": hashlib.sha256(declaration).hexdigest(),
                "public_payload_sha256": hashlib.sha256(payload).hexdigest(),
                "source_profile_sha256": hashlib.sha256(source_profile).hexdigest(),
                "signed_manifest_sha256": hashlib.sha256(signed_manifest).hexdigest(),
                "source_envelope_sha256": hashlib.sha256(source_envelope).hexdigest(),
                "source_envelope_hash": envelope_hash.hex(),
                "ledger_prefix_hash": ledger_prefix_hash.hex(),
                "manifest_digest": manifest_digest.hex(),
                "proof_bundle_sha256": hashlib.sha256(proof).hexdigest(),
            },
            "receipts": [receipt.hex() for receipt in receipts],
            "proof_bundle": proof.hex(),
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    content = json.dumps(generate(), indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(content, encoding="utf-8")
    else:
        print(content, end="")


if __name__ == "__main__":
    main()
