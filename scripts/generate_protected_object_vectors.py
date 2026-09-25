#!/usr/bin/env python3
"""Generate independent Issue #9 protected-object conformance vectors.

The implementation uses only the Python standard library plus the independent
curve/signature primitives from the Issue #8 generator. Every secret below is
a fixed, labelled test fixture; no production custody material is accepted.
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
from pathlib import Path
from typing import Any

from generate_privacy_lifecycle_vectors import (
    P256_N,
    dhash,
    ed25519_public,
    ed25519_sign,
    fingerprint,
    p256_mul,
    p256_public,
    participant_reference,
    record,
)


SUITE = b"ProtectedDataSuiteV1"
NETWORK_ID = "provchain.issue9.vectors"
PROFILE_ID = "privacy.protected.v1"
PROFILE_HASH = bytes([0xA6]) * 32
PRINCIPAL = bytes.fromhex("00112233445566778899aabbccddeeff")
OBJECT_ID = bytes.fromhex("91000000000040018000000000000009")
BOOTSTRAP_SEED = bytes([0x11]) * 32
AUTHORIZATION_SEED = bytes([0x22]) * 32
WRAPPING_SCALAR = bytes([0x33]) * 32
DEK = bytes([0x44]) * 32
HPKE_SEED = bytes([0x55]) * 32
PROTECTED_CONTENT = b"issue-9 protected vector\x00\xff"
P256_P = int(
    "FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF", 16
)


def hkdf_extract(salt: bytes, ikm: bytes) -> bytes:
    return hmac.new(salt, ikm, hashlib.sha256).digest()


def hkdf_expand(prk: bytes, info: bytes, length: int) -> bytes:
    result = bytearray()
    previous = b""
    counter = 1
    while len(result) < length:
        previous = hmac.new(
            prk, previous + info + bytes((counter,)), hashlib.sha256
        ).digest()
        result.extend(previous)
        counter += 1
    return bytes(result[:length])


def rotate_left(value: int, amount: int) -> int:
    return ((value << amount) | (value >> (32 - amount))) & 0xFFFFFFFF


def quarter_round(state: list[int], a: int, b: int, c: int, d: int) -> None:
    state[a] = (state[a] + state[b]) & 0xFFFFFFFF
    state[d] = rotate_left(state[d] ^ state[a], 16)
    state[c] = (state[c] + state[d]) & 0xFFFFFFFF
    state[b] = rotate_left(state[b] ^ state[c], 12)
    state[a] = (state[a] + state[b]) & 0xFFFFFFFF
    state[d] = rotate_left(state[d] ^ state[a], 8)
    state[c] = (state[c] + state[d]) & 0xFFFFFFFF
    state[b] = rotate_left(state[b] ^ state[c], 7)


def chacha20_block(key: bytes, counter: int, nonce: bytes) -> bytes:
    assert len(key) == 32 and len(nonce) == 12
    constants = b"expand 32-byte k"
    initial = [
        int.from_bytes(constants[index : index + 4], "little")
        for index in range(0, 16, 4)
    ]
    initial += [
        int.from_bytes(key[index : index + 4], "little")
        for index in range(0, 32, 4)
    ]
    initial += [counter]
    initial += [
        int.from_bytes(nonce[index : index + 4], "little")
        for index in range(0, 12, 4)
    ]
    working = initial.copy()
    for _ in range(10):
        quarter_round(working, 0, 4, 8, 12)
        quarter_round(working, 1, 5, 9, 13)
        quarter_round(working, 2, 6, 10, 14)
        quarter_round(working, 3, 7, 11, 15)
        quarter_round(working, 0, 5, 10, 15)
        quarter_round(working, 1, 6, 11, 12)
        quarter_round(working, 2, 7, 8, 13)
        quarter_round(working, 3, 4, 9, 14)
    return b"".join(
        ((working[index] + initial[index]) & 0xFFFFFFFF).to_bytes(4, "little")
        for index in range(16)
    )


def chacha20_encrypt(key: bytes, nonce: bytes, plaintext: bytes) -> bytes:
    result = bytearray()
    for block_index, offset in enumerate(range(0, len(plaintext), 64), 1):
        block = chacha20_block(key, block_index, nonce)
        chunk = plaintext[offset : offset + 64]
        result.extend(left ^ right for left, right in zip(chunk, block))
    return bytes(result)


def poly1305(message: bytes, one_time_key: bytes) -> bytes:
    r = int.from_bytes(one_time_key[:16], "little")
    r &= 0x0FFFFFFC0FFFFFFC0FFFFFFC0FFFFFFF
    s = int.from_bytes(one_time_key[16:], "little")
    accumulator = 0
    modulus = (1 << 130) - 5
    for offset in range(0, len(message), 16):
        chunk = message[offset : offset + 16]
        value = int.from_bytes(chunk + b"\x01", "little")
        accumulator = ((accumulator + value) * r) % modulus
    return ((accumulator + s) % (1 << 128)).to_bytes(16, "little")


def pad16(value: bytes) -> bytes:
    return b"" if len(value) % 16 == 0 else bytes(16 - len(value) % 16)


def chacha20_poly1305_seal(
    key: bytes, nonce: bytes, plaintext: bytes, aad: bytes
) -> bytes:
    ciphertext = chacha20_encrypt(key, nonce, plaintext)
    mac_input = (
        aad
        + pad16(aad)
        + ciphertext
        + pad16(ciphertext)
        + len(aad).to_bytes(8, "little")
        + len(ciphertext).to_bytes(8, "little")
    )
    one_time_key = chacha20_block(key, 0, nonce)[:32]
    return ciphertext + poly1305(mac_input, one_time_key)


def run_known_answer_self_tests() -> None:
    hkdf_okm = hkdf_expand(
        hkdf_extract(bytes.fromhex("000102030405060708090a0b0c"), bytes([0x0B]) * 22),
        bytes.fromhex("f0f1f2f3f4f5f6f7f8f9"),
        42,
    )
    assert hkdf_okm.hex() == (
        "3cb25f25faacd57a90434f64d0362f2a"
        "2d2d0a90cf1a5a4c5db02d56ecc4c5bf"
        "34007208d5b887185865"
    )
    assert hmac.new(bytes([0x0B]) * 20, b"Hi There", hashlib.sha256).hexdigest() == (
        "b0344c61d8db38535ca8afceaf0bf12b"
        "881dc200c9833da726e9376c2e32cff7"
    )
    rfc_plaintext = (
        b"Ladies and Gentlemen of the class of '99: If I could offer you only one "
        b"tip for the future, sunscreen would be it."
    )
    rfc_sealed = chacha20_poly1305_seal(
        bytes(range(0x80, 0xA0)),
        bytes.fromhex("070000004041424344454647"),
        rfc_plaintext,
        bytes.fromhex("50515253c0c1c2c3c4c5c6c7"),
    )
    assert rfc_sealed.hex() == (
        "d31a8d34648e60db7b86afbc53ef7ec2"
        "a4aded51296e08fea9e2b5a736ee62d6"
        "3dbea45e8ca9671282fafb69da92728b"
        "1a71de0a9e060b2905d6a5b67ecd3b36"
        "92ddbd7f2d778b8c9803aee328091b58"
        "fab324e4fad675945585808b4831d7bc"
        "3ff4def08e4b7a9de576d26586cec64b"
        "61161ae10b594f09e26a7e902ecbd0600691"
    )


def labeled_extract(suite_id: bytes, salt: bytes, label: bytes, ikm: bytes) -> bytes:
    return hkdf_extract(salt, b"HPKE-v1" + suite_id + label + ikm)


def labeled_expand(
    suite_id: bytes, prk: bytes, label: bytes, info: bytes, length: int
) -> bytes:
    labeled_info = (
        length.to_bytes(2, "big") + b"HPKE-v1" + suite_id + label + info
    )
    return hkdf_expand(prk, labeled_info, length)


def derive_p256_keypair(ikm: bytes) -> tuple[int, bytes]:
    suite_id = b"KEM" + (0x0010).to_bytes(2, "big")
    dkp_prk = labeled_extract(suite_id, b"", b"dkp_prk", ikm)
    for counter in range(256):
        candidate = labeled_expand(
            suite_id, dkp_prk, b"candidate", bytes((counter,)), 32
        )
        scalar = int.from_bytes(candidate, "big")
        if 1 <= scalar < P256_N:
            return scalar, p256_public(candidate)
    raise AssertionError("P-256 test fixture key derivation exhausted")


def p256_decode(public_key: bytes) -> tuple[int, int]:
    assert len(public_key) == 65 and public_key[0] == 4
    point = (int.from_bytes(public_key[1:33], "big"), int.from_bytes(public_key[33:], "big"))
    assert (point[1] * point[1] - (point[0] ** 3 - 3 * point[0] + int(
        "5AC635D8AA3A93E7B3EBBD55769886BC651D06B0CC53B0F63BCE3C3E27D2604B", 16
    ))) % P256_P == 0
    return point


def hpke_seal(recipient_public: bytes, info: bytes, plaintext: bytes) -> tuple[bytes, bytes, bytes]:
    rng_ikm = hmac.new(
        HPKE_SEED,
        b"provchain/protected-data/hpke-csprng-expand/v1" + (0).to_bytes(8, "big"),
        hashlib.sha256,
    ).digest()
    ephemeral_scalar, encapsulated = derive_p256_keypair(rng_ikm)
    shared_point = p256_mul(p256_decode(recipient_public), ephemeral_scalar)
    dh = shared_point[0].to_bytes(32, "big")
    kem_suite = b"KEM" + (0x0010).to_bytes(2, "big")
    kem_context = encapsulated + recipient_public
    eae_prk = labeled_extract(kem_suite, b"", b"eae_prk", dh)
    shared_secret = labeled_expand(
        kem_suite, eae_prk, b"shared_secret", kem_context, 32
    )

    hpke_suite = (
        b"HPKE"
        + (0x0010).to_bytes(2, "big")
        + (0x0001).to_bytes(2, "big")
        + (0x0003).to_bytes(2, "big")
    )
    psk_id_hash = labeled_extract(hpke_suite, b"", b"psk_id_hash", b"")
    info_hash = labeled_extract(hpke_suite, b"", b"info_hash", info)
    context = b"\x00" + psk_id_hash + info_hash
    secret = labeled_extract(hpke_suite, shared_secret, b"secret", b"")
    key = labeled_expand(hpke_suite, secret, b"key", context, 32)
    nonce = labeled_expand(hpke_suite, secret, b"base_nonce", context, 12)
    wrapped = chacha20_poly1305_seal(key, nonce, plaintext, b"")
    return rng_ikm, encapsulated, wrapped


def parent_reference(parent: bytes | None) -> bytes:
    return bytes(33) if parent is None else b"\x01" + parent


def anchor_fields(
    position: int, revision: int, ledger_prefix: bytes, envelope: bytes | None
) -> list[bytes]:
    return [
        NETWORK_ID.encode("ascii"),
        PROFILE_ID.encode("ascii"),
        PROFILE_HASH,
        position.to_bytes(8, "big"),
        revision.to_bytes(8, "big"),
        ledger_prefix,
        parent_reference(envelope),
    ]


def build_vector() -> dict[str, Any]:
    run_known_answer_self_tests()
    authorization_public = ed25519_public(AUTHORIZATION_SEED)
    wrapping_public = p256_public(WRAPPING_SCALAR)
    authorization_fingerprint = fingerprint(
        "PrivacyAuthorizationEd25519V1", 0x01, 0x01, authorization_public
    )
    wrapping_fingerprint = fingerprint(
        "ProtectedDataSuiteV1", 0x02, 0x02, wrapping_public
    )
    authorization_reference = participant_reference(
        PRINCIPAL, 0x01, 1, 0x01, 0x01, authorization_fingerprint
    )
    wrapping_reference = participant_reference(
        PRINCIPAL, 0x02, 1, 0x02, 0x02, wrapping_fingerprint
    )
    anchor = anchor_fields(2, 3, bytes([0xB2]) * 32, bytes([0xC2]) * 32)

    extract_salt = hashlib.sha256(
        b"provchain/protected-data/dek-extract-salt/v1"
    ).digest()
    prk = hkdf_extract(extract_salt, DEK)
    pcc_info = record(
        0x02,
        [b"provchain/protected-data/pcc-key/v1", SUITE],
    )
    pcc_key = hkdf_expand(prk, pcc_info, 32)
    mac_input = record(
        0x04, [b"provchain/protected-data/pcc/v1", PROTECTED_CONTENT]
    )
    pcc = hmac.new(pcc_key, mac_input, hashlib.sha256).digest()
    context_record = record(
        0x20,
        [
            b"PrivacyControlV1",
            b"CanonicalPrivacyEncodingV1",
            b"\x04",
            *anchor,
            OBJECT_ID,
            PRINCIPAL,
            wrapping_reference,
            b"\x21",
            SUITE,
            b"\x23",
            SUITE,
            pcc,
        ],
    )
    object_context = dhash(
        "provchain/protected-object/encryption-context/v1", context_record
    )
    payload_info = record(
        0x03,
        [b"provchain/protected-data/payload-key/v1", SUITE, object_context],
    )
    payload_key = hkdf_expand(prk, payload_info, 32)
    ciphertext = chacha20_poly1305_seal(
        payload_key, bytes(12), PROTECTED_CONTENT, object_context
    )
    payload = record(0x21, [SUITE, bytes(12), pcc, ciphertext])
    epc = dhash(
        "provchain/protected-data/encrypted-payload-commitment/v1", payload
    )
    owner_header = record(
        0x22,
        [
            SUITE,
            anchor[0],
            anchor[1],
            anchor[2],
            anchor[3],
            anchor[4],
            anchor[5],
            anchor[6],
            OBJECT_ID,
            PRINCIPAL,
            wrapping_reference,
            pcc,
            epc,
        ],
    )
    header_hash = dhash(
        "provchain/protected-data/hpke-owner-header/v1", owner_header
    )
    owner_info = object_context + header_hash
    hpke_ikm, encapsulated, wrapped = hpke_seal(
        wrapping_public, owner_info, DEK
    )
    owner_envelope = record(
        0x23, [SUITE, object_context, owner_header, encapsulated, wrapped]
    )
    core = record(
        0x33,
        [
            b"PrivacyControlV1",
            b"CanonicalPrivacyEncodingV1",
            b"\x04",
            *anchor,
            OBJECT_ID,
            PRINCIPAL,
            authorization_reference,
            wrapping_reference,
            object_context,
            payload,
            epc,
            owner_envelope,
            b"\x02",
        ],
    )
    transition_id = dhash(
        "provchain/protected-object/create-transition-id/v1", core
    )
    authorization_digest = dhash(
        "provchain/protected-object/owner-authorization/v1", transition_id + core
    )
    authorization_signature = ed25519_sign(
        AUTHORIZATION_SEED, authorization_digest
    )
    complete = record(0x43, [core, transition_id, authorization_signature])

    tampered_context = bytearray(complete)
    context_offset = complete.find(object_context)
    assert context_offset >= 0
    tampered_context[context_offset] ^= 1
    invalid_enc = bytearray(owner_envelope)
    enc_offset = owner_envelope.find(encapsulated)
    assert enc_offset >= 0
    invalid_enc[enc_offset : enc_offset + 65] = bytes(65)
    invalid_envelope_core = core.replace(owner_envelope, bytes(invalid_enc), 1)
    invalid_envelope_complete = record(
        0x43, [invalid_envelope_core, transition_id, authorization_signature]
    )

    source_path = Path(__file__)
    lock_path = source_path.parent.parent / "Cargo.lock"
    return {
        "inputs": {
            "network_id": NETWORK_ID,
            "profile_id": PROFILE_ID,
            "profile_content_hash_hex": PROFILE_HASH.hex(),
            "principal_uuid": "00112233-4455-6677-8899-aabbccddeeff",
            "object_uuid": "91000000-0000-4001-8000-000000000009",
            "bootstrap_seed_hex": BOOTSTRAP_SEED.hex(),
            "authorization_seed_hex": AUTHORIZATION_SEED.hex(),
            "wrapping_private_scalar_hex": WRAPPING_SCALAR.hex(),
            "dek_fixture_hex": DEK.hex(),
            "hpke_seed_fixture_hex": HPKE_SEED.hex(),
            "protected_content_hex": PROTECTED_CONTENT.hex(),
        },
        "create_protected_object": {
            "authorization_public_key_hex": authorization_public.hex(),
            "wrapping_public_key_hex": wrapping_public.hex(),
            "authorization_reference_hex": authorization_reference.hex(),
            "wrapping_reference_hex": wrapping_reference.hex(),
            "pcc_hex": pcc.hex(),
            "object_context_hex": object_context.hex(),
            "payload_hex": payload.hex(),
            "epc_hex": epc.hex(),
            "owner_header_hex": owner_header.hex(),
            "owner_info_hex": owner_info.hex(),
            "hpke_ikm_hex": hpke_ikm.hex(),
            "encapsulated_key_hex": encapsulated.hex(),
            "wrapped_dek_hex": wrapped.hex(),
            "owner_envelope_hex": owner_envelope.hex(),
            "core_hex": core.hex(),
            "transition_id_hex": transition_id.hex(),
            "authorization_digest_hex": authorization_digest.hex(),
            "authorization_signature_hex": authorization_signature.hex(),
            "complete_hex": complete.hex(),
        },
        "negative": {
            "tampered_context_complete_hex": bytes(tampered_context).hex(),
            "invalid_encapsulated_point_complete_hex": invalid_envelope_complete.hex(),
            "truncated_complete_hex": complete[:-1].hex(),
        },
        "manifest": {
            "schema": "ProvChainProtectedObjectVectorsV1",
            "scope": "Issue #9 CreateProtectedObject and owner-envelope construction only",
            "generator": {
                "source": "scripts/generate_protected_object_vectors.py",
                "implementation": "independent Python standard-library reference",
                "generate_command": "python3 scripts/generate_protected_object_vectors.py --output tests/vectors/protected_object_v1.json",
                "verify_command": "python3 scripts/generate_protected_object_vectors.py --verify tests/vectors/protected_object_v1.json",
                "source_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
            },
            "cargo_lock_sha256": hashlib.sha256(lock_path.read_bytes()).hexdigest(),
            "rust_msrv": "1.87",
            "protocol": "PrivacyControlV1",
            "canonical_encoding": "CanonicalPrivacyEncodingV1",
            "protected_data_suite": "ProtectedDataSuiteV1",
            "rust_dependencies": {
                "argon2": {
                    "version": "0.5.3",
                    "default_features": False,
                    "features": ["zeroize"],
                    "checksum": "3c3610892ee6e0cbce8ae2700349fcf8f98adb0dbfbee85aec3c9179d29cc072",
                },
                "chacha20poly1305": {
                    "version": "0.11.0",
                    "default_features": False,
                    "features": ["alloc", "zeroize"],
                    "checksum": "9b89e1c441e926b9c82a8d023f6e1b7ae0adcfaa7d621814e4d60789bac751cb",
                },
                "ed25519-dalek": {
                    "version": "2.2.0",
                    "default_features": False,
                    "features": ["fast", "zeroize"],
                    "checksum": "70e796c081cee67dc755e1a36a0a172b897fab85fc3f6bc48307991f64e4eca9",
                },
                "getrandom": {
                    "version": "0.4.3",
                    "default_features": False,
                    "features": ["sys_rng"],
                    "checksum": "300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099",
                },
                "hkdf": {
                    "version": "0.13.0",
                    "default_features": False,
                    "features": [],
                    "checksum": "4aaa26c720c68b866f2c96ef5c1264b3e6f473fe5d4ce61cd44bbe913e553018",
                },
                "hmac": {
                    "version": "0.13.0",
                    "default_features": False,
                    "features": [],
                    "checksum": "6303bc9732ae41b04cb554b844a762b4115a61bfaa81e3e83050991eeb56863f",
                },
                "hpke": {
                    "version": "0.14.0",
                    "default_features": False,
                    "features": ["alloc", "chacha", "nistp"],
                    "checksum": "dd5130e119706b4d8c2180da6126f7e60b6c38c2d340d539219f57051f0a7af7",
                },
                "p256": {
                    "version": "0.14.0",
                    "default_features": False,
                    "features": ["ecdsa"],
                    "checksum": "d2c9239b2dbc807adbbe147e8cf72ea7450c3a0aabe62cb8e75ff4ec22e1f72a",
                },
                "rand_core": {
                    "version": "0.10.1",
                    "default_features": False,
                    "features": [],
                    "checksum": "63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69",
                },
                "rustix": {
                    "version": "1.1.3",
                    "default_features": False,
                    "features": ["fs", "process", "std"],
                    "checksum": "146c9e247ccc180c1f61615433868c99f3de3ae256a30a43b49f67c2d9171f34",
                },
                "sha2": {
                    "version": "0.11.0",
                    "default_features": False,
                    "features": [],
                    "checksum": "446ba717509524cb3f22f17ecc096f10f4822d76ab5c0b9822c5f9c284e825f4",
                },
                "zeroize": {
                    "version": "1.8.2",
                    "default_features": False,
                    "features": [],
                    "checksum": "b97154e67e32c85465826e8bcc1c59429aaaf107c1e4a9e53c8d8ccd5eff88d0",
                },
            },
            "known_answer_tests": [
                "RFC5869-HKDF-SHA256-case-1",
                "RFC4231-HMAC-SHA256-case-1",
                "RFC8439-ChaCha20Poly1305-section-2.8.2",
            ],
        },
    }


def encoded(vector: dict[str, Any]) -> str:
    return json.dumps(vector, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    output = encoded(build_vector())
    if args.verify is not None:
        if not args.verify.is_file() or args.verify.read_text() != output:
            raise SystemExit(f"vector mismatch: {args.verify}")
        print(f"verified {args.verify}")
        return 0
    if args.output is not None:
        args.output.write_text(output, encoding="utf-8")
        print(f"wrote {args.output}")
        return 0
    print(output, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
