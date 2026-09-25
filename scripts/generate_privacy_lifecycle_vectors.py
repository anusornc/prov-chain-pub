#!/usr/bin/env python3
"""Generate independent Issue #8 privacy-lifecycle conformance vectors.

This reference implementation intentionally uses only the Python standard
library and does not import ProvChain or any Rust crypto dependency.  All
private values below are fixed, labelled test fixtures.
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
from pathlib import Path
from typing import Any


RECORD_MAGIC = b"PCV1"
P256_P = int("FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF", 16)
P256_A = P256_P - 3
P256_B = int("5AC635D8AA3A93E7B3EBBD55769886BC651D06B0CC53B0F63BCE3C3E27D2604B", 16)
P256_N = int("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551", 16)
P256_G = (
    int("6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296", 16),
    int("4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5", 16),
)
ED25519_Q = 2**255 - 19
ED25519_L = 2**252 + 27742317777372353535851937790883648493
ED25519_D = (-121665 * pow(121666, ED25519_Q - 2, ED25519_Q)) % ED25519_Q
ED25519_I = pow(2, (ED25519_Q - 1) // 4, ED25519_Q)


def record(tag: int, fields: list[bytes]) -> bytes:
    encoded = bytearray(RECORD_MAGIC + bytes((tag, len(fields))))
    for index, field in enumerate(fields, 1):
        encoded.extend(bytes((index,)))
        encoded.extend(len(field).to_bytes(4, "big"))
        encoded.extend(field)
    return bytes(encoded)


def dhash(domain: str, payload: bytes) -> bytes:
    return hashlib.sha256(record(0x01, [domain.encode("ascii"), payload])).digest()


def fingerprint(scheme: str, algorithm: int, encoding: int, public_key: bytes) -> bytes:
    fingerprint_input = record(
        0x05,
        [scheme.encode("ascii"), bytes((algorithm,)), bytes((encoding,)), public_key],
    )
    return dhash("provchain/privacy-key/fingerprint/v1", fingerprint_input)


def ed25519_xrecover(y: int) -> int:
    xx = (y * y - 1) * pow(ED25519_D * y * y + 1, ED25519_Q - 2, ED25519_Q)
    x = pow(xx % ED25519_Q, (ED25519_Q + 3) // 8, ED25519_Q)
    if (x * x - xx) % ED25519_Q != 0:
        x = x * ED25519_I % ED25519_Q
    if x & 1:
        x = ED25519_Q - x
    return x


ED25519_BASE_Y = 4 * pow(5, ED25519_Q - 2, ED25519_Q) % ED25519_Q
ED25519_BASE = (ed25519_xrecover(ED25519_BASE_Y), ED25519_BASE_Y)


def ed25519_add(left: tuple[int, int], right: tuple[int, int]) -> tuple[int, int]:
    x1, y1 = left
    x2, y2 = right
    product = ED25519_D * x1 * x2 * y1 * y2 % ED25519_Q
    x3 = (x1 * y2 + x2 * y1) * pow(1 + product, ED25519_Q - 2, ED25519_Q)
    y3 = (y1 * y2 + x1 * x2) * pow(1 - product, ED25519_Q - 2, ED25519_Q)
    return x3 % ED25519_Q, y3 % ED25519_Q


def ed25519_mul(point: tuple[int, int], scalar: int) -> tuple[int, int]:
    result = (0, 1)
    addend = point
    while scalar:
        if scalar & 1:
            result = ed25519_add(result, addend)
        addend = ed25519_add(addend, addend)
        scalar >>= 1
    return result


def ed25519_encode(point: tuple[int, int]) -> bytes:
    x, y = point
    encoded = y | ((x & 1) << 255)
    return encoded.to_bytes(32, "little")


def ed25519_expand(seed: bytes) -> tuple[int, bytes]:
    digest = hashlib.sha512(seed).digest()
    scalar_bytes = bytearray(digest[:32])
    scalar_bytes[0] &= 248
    scalar_bytes[31] &= 63
    scalar_bytes[31] |= 64
    return int.from_bytes(scalar_bytes, "little"), digest[32:]


def ed25519_public(seed: bytes) -> bytes:
    scalar, _ = ed25519_expand(seed)
    return ed25519_encode(ed25519_mul(ED25519_BASE, scalar))


def ed25519_sign(seed: bytes, message: bytes) -> bytes:
    scalar, prefix = ed25519_expand(seed)
    public_key = ed25519_encode(ed25519_mul(ED25519_BASE, scalar))
    nonce = int.from_bytes(hashlib.sha512(prefix + message).digest(), "little") % ED25519_L
    encoded_r = ed25519_encode(ed25519_mul(ED25519_BASE, nonce))
    challenge = int.from_bytes(
        hashlib.sha512(encoded_r + public_key + message).digest(), "little"
    ) % ED25519_L
    encoded_s = ((nonce + challenge * scalar) % ED25519_L).to_bytes(32, "little")
    return encoded_r + encoded_s


def p256_add(
    left: tuple[int, int] | None, right: tuple[int, int] | None
) -> tuple[int, int] | None:
    if left is None:
        return right
    if right is None:
        return left
    x1, y1 = left
    x2, y2 = right
    if x1 == x2 and (y1 + y2) % P256_P == 0:
        return None
    if left == right:
        slope = (3 * x1 * x1 + P256_A) * pow(2 * y1, P256_P - 2, P256_P)
    else:
        slope = (y2 - y1) * pow((x2 - x1) % P256_P, P256_P - 2, P256_P)
    slope %= P256_P
    x3 = (slope * slope - x1 - x2) % P256_P
    return x3, (slope * (x1 - x3) - y1) % P256_P


def p256_mul(point: tuple[int, int], scalar: int) -> tuple[int, int]:
    result = None
    addend: tuple[int, int] | None = point
    while scalar:
        if scalar & 1:
            result = p256_add(result, addend)
        addend = p256_add(addend, addend)
        scalar >>= 1
    assert result is not None
    return result


def p256_public(private_scalar: bytes) -> bytes:
    x, y = p256_mul(P256_G, int.from_bytes(private_scalar, "big"))
    return b"\x04" + x.to_bytes(32, "big") + y.to_bytes(32, "big")


def bits2octets(digest: bytes) -> bytes:
    value = int.from_bytes(digest, "big")
    if value >= P256_N:
        value -= P256_N
    return value.to_bytes(32, "big")


def rfc6979_nonce(private_scalar: bytes, digest: bytes) -> int:
    value = b"\x01" * 32
    key = b"\x00" * 32
    seed = private_scalar + bits2octets(digest)
    key = hmac.new(key, value + b"\x00" + seed, hashlib.sha256).digest()
    value = hmac.new(key, value, hashlib.sha256).digest()
    key = hmac.new(key, value + b"\x01" + seed, hashlib.sha256).digest()
    value = hmac.new(key, value, hashlib.sha256).digest()
    while True:
        value = hmac.new(key, value, hashlib.sha256).digest()
        candidate = int.from_bytes(value, "big")
        if 1 <= candidate < P256_N:
            return candidate
        key = hmac.new(key, value + b"\x00", hashlib.sha256).digest()
        value = hmac.new(key, value, hashlib.sha256).digest()


def p256_sign(private_scalar: bytes, message: bytes, *, prehashed: bool = False) -> bytes:
    digest = message if prehashed else hashlib.sha256(message).digest()
    nonce = rfc6979_nonce(private_scalar, digest)
    r = p256_mul(P256_G, nonce)[0] % P256_N
    private_value = int.from_bytes(private_scalar, "big")
    s = pow(nonce, -1, P256_N) * (int.from_bytes(digest, "big") + r * private_value)
    s %= P256_N
    if s > P256_N // 2:
        s = P256_N - s
    assert 1 <= r < P256_N and 1 <= s <= P256_N // 2
    return r.to_bytes(32, "big") + s.to_bytes(32, "big")


def der_integer(value: int) -> bytes:
    encoded = value.to_bytes((value.bit_length() + 7) // 8 or 1, "big")
    if encoded[0] & 0x80:
        encoded = b"\x00" + encoded
    return b"\x02" + bytes((len(encoded),)) + encoded


def p256_der(raw_signature: bytes) -> bytes:
    encoded = der_integer(int.from_bytes(raw_signature[:32], "big")) + der_integer(
        int.from_bytes(raw_signature[32:], "big")
    )
    return b"\x30" + bytes((len(encoded),)) + encoded


def participant_binding(
    principal: bytes,
    purpose: int,
    version: int,
    algorithm: int,
    encoding: int,
    possession_scheme: int,
    scheme: str,
    public_key: bytes,
) -> tuple[bytes, bytes]:
    key_fingerprint = fingerprint(scheme, algorithm, encoding, public_key)
    binding = record(
        0x10,
        [
            principal,
            bytes((purpose,)),
            version.to_bytes(4, "big"),
            bytes((algorithm,)),
            bytes((encoding,)),
            key_fingerprint,
            bytes((possession_scheme,)),
            public_key,
        ],
    )
    return binding, key_fingerprint


def participant_reference(
    principal: bytes,
    purpose: int,
    version: int,
    algorithm: int,
    encoding: int,
    key_fingerprint: bytes,
) -> bytes:
    return record(
        0x11,
        [
            principal,
            bytes((purpose,)),
            version.to_bytes(4, "big"),
            bytes((algorithm,)),
            bytes((encoding,)),
            key_fingerprint,
        ],
    )


def anchor_fields(
    network_id: str,
    profile_id: str,
    profile_hash: bytes,
    position: int,
    revision: int,
    parent_prefix: bytes,
    parent_envelope: bytes | None,
) -> list[bytes]:
    parent_reference = (b"\x00" + bytes(32)) if parent_envelope is None else b"\x01" + parent_envelope
    return [
        network_id.encode("ascii"),
        profile_id.encode("ascii"),
        profile_hash,
        position.to_bytes(8, "big"),
        revision.to_bytes(8, "big"),
        parent_prefix,
        parent_reference,
    ]


def common_fields(variant: int, anchor: list[bytes]) -> list[bytes]:
    return [
        b"PrivacyControlV1",
        b"CanonicalPrivacyEncodingV1",
        bytes((variant,)),
        *anchor,
    ]


def transition_values(
    core: bytes, transition_domain: str, authorization_domain: str, possession_domain: str | None
) -> tuple[bytes, bytes, bytes | None]:
    transition_id = dhash(transition_domain, core)
    proof_payload = transition_id + core
    authorization = dhash(authorization_domain, proof_payload)
    possession = dhash(possession_domain, proof_payload) if possession_domain else None
    return transition_id, authorization, possession


def hex_value(value: bytes) -> str:
    return value.hex()


def build_vectors() -> dict[str, Any]:
    repository = Path(__file__).resolve().parents[1]
    generator_hash = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    lock_hash = hashlib.sha256((repository / "Cargo.lock").read_bytes()).hexdigest()

    bootstrap_seed = bytes((0x11,)) * 32
    authorization_seed = bytes((0x22,)) * 32
    wrapping_scalar = bytes((0x33,)) * 32
    bootstrap_public = ed25519_public(bootstrap_seed)
    authorization_public = ed25519_public(authorization_seed)
    wrapping_public = p256_public(wrapping_scalar)
    principal = bytes.fromhex("00112233445566778899aabbccddeeff")
    network_id = "provchain.issue8.vectors"
    profile_id = "privacy.lifecycle.v1"
    profile_hash = bytes((0xA5,)) * 32

    register_anchor = anchor_fields(
        network_id, profile_id, profile_hash, 0, 1, bytes((0xB0,)) * 32, None
    )
    bind_anchor = anchor_fields(
        network_id,
        profile_id,
        profile_hash,
        1,
        2,
        bytes((0xB1,)) * 32,
        bytes((0xC1,)) * 32,
    )
    revoke_anchor = anchor_fields(
        network_id,
        profile_id,
        profile_hash,
        2,
        3,
        bytes((0xB2,)) * 32,
        bytes((0xC2,)) * 32,
    )

    authorization_binding, authorization_fingerprint = participant_binding(
        principal,
        0x01,
        1,
        0x01,
        0x01,
        0x01,
        "PrivacyAuthorizationEd25519V1",
        authorization_public,
    )
    bootstrap_fingerprint = fingerprint(
        "PrivacyBootstrapGovernanceEd25519V1", 0x01, 0x01, bootstrap_public
    )
    bootstrap_reference = record(
        0x12,
        [b"\x01", (1).to_bytes(4, "big"), b"\x01", b"\x01", bootstrap_fingerprint],
    )
    register_core = record(
        0x30,
        common_fields(0x01, register_anchor)
        + [principal, authorization_binding, b"\x01", bootstrap_reference],
    )
    register_id, register_authorization, register_possession = transition_values(
        register_core,
        "provchain/privacy-bootstrap/transition-id/v1",
        "provchain/privacy-bootstrap/governance-authorization/v1",
        "provchain/privacy-bootstrap/key-possession/v1",
    )
    assert register_possession is not None
    governance_signature = ed25519_sign(bootstrap_seed, register_authorization)
    registration_possession_signature = ed25519_sign(authorization_seed, register_possession)
    register_complete = record(
        0x40,
        [register_core, register_id, governance_signature, registration_possession_signature],
    )

    authorization_reference = participant_reference(
        principal, 0x01, 1, 0x01, 0x01, authorization_fingerprint
    )
    wrapping_binding, wrapping_fingerprint = participant_binding(
        principal,
        0x02,
        1,
        0x02,
        0x02,
        0x02,
        "ProtectedDataSuiteV1",
        wrapping_public,
    )
    bind_core = record(
        0x31,
        common_fields(0x02, bind_anchor)
        + [principal, wrapping_binding, b"\x02", authorization_reference],
    )
    bind_id, bind_authorization, bind_possession = transition_values(
        bind_core,
        "provchain/privacy-key-bind/transition-id/v1",
        "provchain/privacy-key-bind/participant-authorization/v1",
        "provchain/privacy-key-bind/key-possession/v1",
    )
    assert bind_possession is not None
    bind_authorization_signature = ed25519_sign(authorization_seed, bind_authorization)
    bind_possession_proof = p256_sign(wrapping_scalar, bind_possession)
    bind_complete = record(
        0x41,
        [bind_core, bind_id, bind_authorization_signature, bind_possession_proof],
    )

    revoke_core = record(
        0x32,
        common_fields(0x03, revoke_anchor)
        + [
            principal,
            principal,
            b"\x02",
            (1).to_bytes(4, "big"),
            b"\x02",
            authorization_reference,
        ],
    )
    revoke_id, revoke_authorization, _ = transition_values(
        revoke_core,
        "provchain/privacy-key-revoke/transition-id/v1",
        "provchain/privacy-key-revoke/participant-authorization/v1",
        None,
    )
    revoke_authorization_signature = ed25519_sign(authorization_seed, revoke_authorization)
    revoke_complete = record(
        0x42, [revoke_core, revoke_id, revoke_authorization_signature]
    )

    r = bind_possession_proof[:32]
    low_s = int.from_bytes(bind_possession_proof[32:], "big")
    high_s_proof = r + (P256_N - low_s).to_bytes(32, "big")
    der_proof = p256_der(bind_possession_proof)
    prehash_proof = p256_sign(wrapping_scalar, bind_possession, prehashed=True)
    zero_r_proof = bytes(32) + bind_possession_proof[32:]

    invalid_point = b"\x04" + bytes(64)
    invalid_binding, _ = participant_binding(
        principal,
        0x02,
        1,
        0x02,
        0x02,
        0x02,
        "ProtectedDataSuiteV1",
        invalid_point,
    )
    invalid_point_core = record(
        0x31,
        common_fields(0x02, bind_anchor)
        + [principal, invalid_binding, b"\x02", authorization_reference],
    )
    invalid_id, invalid_authorization, invalid_possession = transition_values(
        invalid_point_core,
        "provchain/privacy-key-bind/transition-id/v1",
        "provchain/privacy-key-bind/participant-authorization/v1",
        "provchain/privacy-key-bind/key-possession/v1",
    )
    assert invalid_possession is not None
    invalid_point_complete = record(
        0x41,
        [
            invalid_point_core,
            invalid_id,
            ed25519_sign(authorization_seed, invalid_authorization),
            p256_sign(wrapping_scalar, invalid_possession),
        ],
    )

    return {
        "manifest": {
            "schema": "ProvChainPrivacyLifecycleVectorsV1",
            "scope": "Issue #8 key-lifecycle transitions only; protected-object and grant fixtures are tracked separately",
            "canonical_encoding": "CanonicalPrivacyEncodingV1",
            "generator": {
                "implementation": "independent Python standard-library reference",
                "source": "scripts/generate_privacy_lifecycle_vectors.py",
                "source_sha256": generator_hash,
                "generate_command": "python3 scripts/generate_privacy_lifecycle_vectors.py --output tests/vectors/privacy_lifecycle_v1.json",
                "verify_command": "python3 scripts/generate_privacy_lifecycle_vectors.py --verify tests/vectors/privacy_lifecycle_v1.json",
            },
            "rust_msrv": "1.87",
            "cargo_lock_sha256": lock_hash,
            "rust_dependencies": {
                "ed25519-dalek": {
                    "version": "2.2.0",
                    "default_features": False,
                    "features": ["fast", "zeroize"],
                    "checksum": "70e796c081cee67dc755e1a36a0a172b897fab85fc3f6bc48307991f64e4eca9",
                },
                "p256": {
                    "version": "0.14.0",
                    "default_features": False,
                    "features": ["ecdsa"],
                    "checksum": "d2c9239b2dbc807adbbe147e8cf72ea7450c3a0aabe62cb8e75ff4ec22e1f72a",
                },
            },
        },
        "inputs": {
            "network_id": network_id,
            "profile_id": profile_id,
            "profile_content_hash_hex": hex_value(profile_hash),
            "principal_uuid": "00112233-4455-6677-8899-aabbccddeeff",
            "bootstrap_seed_hex": hex_value(bootstrap_seed),
            "bootstrap_public_key_hex": hex_value(bootstrap_public),
            "authorization_seed_hex": hex_value(authorization_seed),
            "authorization_public_key_hex": hex_value(authorization_public),
            "wrapping_private_scalar_hex": hex_value(wrapping_scalar),
            "wrapping_public_key_hex": hex_value(wrapping_public),
            "anchors": {
                "register": {
                    "position": 0,
                    "privacy_revision": 1,
                    "parent_ledger_prefix_hash_hex": hex_value(bytes((0xB0,)) * 32),
                    "parent_envelope_hash_hex": None,
                },
                "bind": {
                    "position": 1,
                    "privacy_revision": 2,
                    "parent_ledger_prefix_hash_hex": hex_value(bytes((0xB1,)) * 32),
                    "parent_envelope_hash_hex": hex_value(bytes((0xC1,)) * 32),
                },
                "revoke": {
                    "position": 2,
                    "privacy_revision": 3,
                    "parent_ledger_prefix_hash_hex": hex_value(bytes((0xB2,)) * 32),
                    "parent_envelope_hash_hex": hex_value(bytes((0xC2,)) * 32),
                },
            },
        },
        "fingerprints": {
            "bootstrap_governance_hex": hex_value(bootstrap_fingerprint),
            "authorization_v1_hex": hex_value(authorization_fingerprint),
            "wrapping_v1_hex": hex_value(wrapping_fingerprint),
        },
        "register": {
            "core_hex": hex_value(register_core),
            "transition_id_hex": hex_value(register_id),
            "authorization_digest_hex": hex_value(register_authorization),
            "possession_digest_hex": hex_value(register_possession),
            "governance_signature_hex": hex_value(governance_signature),
            "possession_signature_hex": hex_value(registration_possession_signature),
            "complete_hex": hex_value(register_complete),
        },
        "bind_wrapping": {
            "core_hex": hex_value(bind_core),
            "transition_id_hex": hex_value(bind_id),
            "authorization_digest_hex": hex_value(bind_authorization),
            "possession_digest_hex": hex_value(bind_possession),
            "authorization_signature_hex": hex_value(bind_authorization_signature),
            "low_s_possession_proof_hex": hex_value(bind_possession_proof),
            "complete_hex": hex_value(bind_complete),
        },
        "revoke_wrapping": {
            "core_hex": hex_value(revoke_core),
            "transition_id_hex": hex_value(revoke_id),
            "authorization_digest_hex": hex_value(revoke_authorization),
            "authorization_signature_hex": hex_value(revoke_authorization_signature),
            "complete_hex": hex_value(revoke_complete),
        },
        "negative": {
            "p256_high_s_proof_hex": hex_value(high_s_proof),
            "p256_high_s_complete_hex": hex_value(
                record(0x41, [bind_core, bind_id, bind_authorization_signature, high_s_proof])
            ),
            "p256_der_proof_hex": hex_value(der_proof),
            "p256_der_complete_hex": hex_value(
                record(0x41, [bind_core, bind_id, bind_authorization_signature, der_proof])
            ),
            "p256_prehash_proof_hex": hex_value(prehash_proof),
            "p256_prehash_complete_hex": hex_value(
                record(0x41, [bind_core, bind_id, bind_authorization_signature, prehash_proof])
            ),
            "p256_zero_r_proof_hex": hex_value(zero_r_proof),
            "p256_zero_r_complete_hex": hex_value(
                record(0x41, [bind_core, bind_id, bind_authorization_signature, zero_r_proof])
            ),
            "invalid_p256_point_hex": hex_value(invalid_point),
            "invalid_p256_point_complete_hex": hex_value(invalid_point_complete),
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", type=Path, help="compare generated vectors with this fixture")
    parser.add_argument("--output", type=Path, help="write generated vectors to this fixture")
    arguments = parser.parse_args()
    generated = build_vectors()
    if arguments.verify is not None:
        expected = json.loads(arguments.verify.read_text(encoding="utf-8"))
        if generated != expected:
            raise SystemExit(f"privacy lifecycle vector mismatch: {arguments.verify}")
        print(f"verified {arguments.verify}")
        return 0
    if arguments.output is not None:
        arguments.output.write_text(
            json.dumps(generated, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        print(f"wrote {arguments.output}")
        return 0
    print(json.dumps(generated, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
