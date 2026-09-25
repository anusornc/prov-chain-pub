# Security Policy

## Reporting Vulnerabilities

If you discover a security vulnerability in ProvChainOrg, please report it responsibly.

### How to Report

1. **Email**: Send a detailed report to the project maintainers
2. **Include**: Steps to reproduce, expected vs actual behavior, and impact assessment
3. **Wait**: Allow time for the maintainers to investigate and respond

### What to Include

- Vulnerability type and severity
- Steps to reproduce the issue
- Potential impact assessment
- Any suggested fixes or mitigations

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution**: As soon as practicable based on severity

### Supported Versions

Security updates are provided for the latest stable release.

## Security Features

ProvChainOrg currently includes the following implemented security primitives and web controls:

- **Ed25519 Digital Signatures**: Block and transaction signing
- **ChaCha-Family Encryption Primitive**: Low-level encrypted-data scaffolding; it is not the
  conforming `ProtectedDataSuiteV1` path
- **JWT Authentication**: Web API tokens use HS256 with `JWT_SECRET` (32+ chars), expire after 24
  hours, and currently have no refresh route
- **One-Time Admin Bootstrap**: `/auth/bootstrap` gated by `PROVCHAIN_BOOTSTRAP_TOKEN`
- **Rate-Limiter Scaffolding**: Middleware and tests exist, but active coverage of every public route
  is not established
- **CORS Policy**: Configurable origin whitelisting
- **Administrative Audit Logging**: User-management actions can be recorded; this is not the pending
  immutable journal-derived security audit

TLS/SSL termination and TLS 1.3 P2P transport are deployment/target controls, not implemented
runtime evidence. Existing configuration, reverse-proxy templates, and certificate-path fields do
not prove that either control is active.

The journal-authoritative privacy architecture in ADRs 0027-0036 is accepted but not implemented or
active. ADR 0034 pins `ProtectedDataSuiteV1`: each object has a fresh per-object DEK, the payload uses
a one-use derived key with ChaCha20Poly1305, an all-zero 12-byte nonce, and the raw Object Encryption
Context Digest `O` as associated data. Encryption or wrapping keys are never derived from an owner
signing key. Under the accepted contract, decryption after a Live Privacy Release is client-side:
the service must return the exact committed ciphertext, exactly one applicable DEK envelope, and
bound release evidence; it must never return plaintext or decrypt on the server.

ADR 0035 fixes a separate durable client-only Participant Key Custody boundary. In a conforming
implementation, nodes, web services, the journal, projections, auth-user storage, and server backups
must never receive participant passphrases or private keys. The client must retain exact Active and
Retired versions, reconcile them against ledger-derived status, and treat missing material as a
local Custody Gap rather than a ledger lifecycle change. ADR 0036 fixes the exact encrypted-keystore
KDF, AEAD, and binary format,
single-writer whole-snapshot commit, exact retry, backup, and quarantined restore contract. That
contract remains unimplemented, so the legacy server `WalletManager` and its plaintext JSON backup
are explicitly nonconforming.

The accepted complete-envelope Ledger Journal is specified as append-only, but its implementation
and evidence remain pending. Even when implemented, that journal will not provide blanket erasure
or a right to be forgotten. Selecting a cryptographic suite does not establish GDPR, FIPS, HSM, or
production-readiness compliance. Participant-key lifecycle is ledger-derived and purpose-specific
rather than a blanket 90-day rotation rule. See
[ADR 0034](docs/architecture/ADR/0034-pin-protected-data-suite-v1-and-canonical-privacy-encoding.md),
[ADR 0035](docs/architecture/ADR/0035-keep-participant-private-keys-in-durable-client-only-custody.md),
and [ADR 0036](docs/architecture/ADR/0036-pin-participant-keystore-suite-v1-and-whole-snapshot-commit.md).

For detailed security documentation, see:
- [Security Setup Guide](docs/security/SECURITY_SETUP.md)
- [Security Test Coverage](docs/security/SECURITY_TEST_COVERAGE_REPORT.md)
- [Security Architecture](docs/architecture/SECURITY_ARCHITECTURE.md)

## Security Best Practices

### For Developers

- Never commit secrets or credentials
- Use environment variables for sensitive configuration
- Enable security features in production mode
- Regularly update dependencies
- Follow the [Contributing Guidelines](CONTRIBUTING.md)

### For Operators

- Use strong JWT secrets (32+ characters)
- Set `JWT_SECRET` via environment (no config-file fallback)
- Set `PROVCHAIN_BOOTSTRAP_TOKEN` only for first-admin provisioning, then rotate/remove it
- Set `PROVCHAIN_AUTH_USERS_FILE` to a protected path such as `./data/auth/users.json` when API users must survive restart
- Terminate TLS at a reviewed deployment boundary and verify it independently; repository templates
  are not activation evidence
- Configure appropriate rate limits
- Set up audit logging
- Monitor security events
- Follow the [Production Deployment Guide](docs/deployment/README.md)

## Dependency Security

Comments in `Cargo.toml` record selected historical advisory assessments; they are not a current,
complete audit result. To produce current dependency evidence, run and archive:

```bash
cargo audit
```

## License

This security policy is part of the ProvChainOrg project.
