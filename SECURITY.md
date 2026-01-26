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

ProvChainOrg includes the following security features:

- **Ed25519 Digital Signatures**: Block and transaction signing
- **ChaCha20-Poly1305 Encryption**: Private data encryption
- **JWT Authentication**: API authentication with configurable secrets
- **TLS/SSL Support**: Secure network communications
- **Rate Limiting**: DoS protection
- **CORS Policy**: Configurable origin whitelisting
- **Audit Logging**: Security event tracking
- **GDPR Compliance**: Right-to-be-forgotten implementation
- **Key Rotation**: 90-day recommended interval

For detailed security documentation, see:
- [Security Setup Guide](docs/security/SECURITY_SETUP.md)
- [Security Test Coverage](docs/security/SECURITY_TEST_COVERAGE_REPORT.md)

## Security Best Practices

### For Developers

- Never commit secrets or credentials
- Use environment variables for sensitive configuration
- Enable security features in production mode
- Regularly update dependencies
- Follow the [Contributing Guidelines](CONTRIBUTING.md)

### For Operators

- Use strong JWT secrets (32+ characters)
- Enable TLS/SSL in production
- Configure appropriate rate limits
- Set up audit logging
- Monitor security events
- Follow the [Production Deployment Guide](docs/deployment/README.md)

## Dependency Security

All security advisories are documented in `Cargo.toml` with risk assessments.

To check for security vulnerabilities:

```bash
cargo audit
```

## License

This security policy is part of the ProvChainOrg project.
