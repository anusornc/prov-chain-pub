# 🔒 ProvChain-Org Security Setup Guide

## Overview

This guide provides step-by-step instructions for securely configuring and deploying the ProvChain-Org system with all security enhancements implemented.

## 🚨 Critical Security Requirements

### 1. JWT Secret Configuration (REQUIRED)

The system now requires a secure JWT secret to be set via environment variable:

```bash
# Generate a cryptographically secure JWT secret (RECOMMENDED)
export JWT_SECRET=$(openssl rand -base64 32)

# OR set a custom secure secret (MINIMUM 32 characters)
export JWT_SECRET="your-super-secret-jwt-key-at-least-32-chars-long"
```

**⚠️ WARNING:** Never commit JWT secrets to version control or use predictable secrets.

### 2. First Admin Bootstrap (REQUIRED)

The system does not create default users. Bootstrap the first admin explicitly:

```bash
# One-time bootstrap token (32+ chars)
export PROVCHAIN_BOOTSTRAP_TOKEN=$(openssl rand -base64 32)

# Create first admin account
curl -X POST http://localhost:8080/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{
    "username":"your-admin",
    "password":"SecurePassword123!",
    "bootstrap_token":"'"$PROVCHAIN_BOOTSTRAP_TOKEN"'"
  }'
```

After any user exists, `/auth/bootstrap` is disabled.

## 🔧 Environment Variables

### Required for Production
```bash
# JWT Authentication
export JWT_SECRET="your-secure-32+character-secret"

# First-admin bootstrap protection
export PROVCHAIN_BOOTSTRAP_TOKEN="one-time-bootstrap-token-32+chars"
```

### Optional Security Enhancements
```bash
# CORS Origins (comma-separated)
export ALLOWED_ORIGINS="https://yourdomain.com,https://app.yourdomain.com"

# Log Level (recommended: warn or error for production)
export PROVCHAIN_LOG_LEVEL="warn"
```

## 🛡️ Security Features Enabled

### 1. Input Validation
- **Username validation**: 3-50 characters, alphanumeric + underscore/hyphen
- **Password validation**:
  - Production: 8+ chars, uppercase, lowercase, digit, special character
  - Development: 6+ chars with letters and numbers
- **RDF/URI validation**: Prevents injection attacks
- **SPARQL query validation**: Blocks dangerous operations

### 2. Rate Limiting
- **Authentication endpoints**: 5 attempts per 5 minutes per IP
- **API endpoints**: 1000 requests per minute per IP
- **Automatic cleanup**: Expired entries removed every 5 minutes

### 3. Security Headers
All responses include:
- `Content-Security-Policy`: Prevents XSS and code injection
- `X-Content-Type-Options`: Prevents MIME sniffing
- `X-Frame-Options`: Prevents clickjacking
- `X-XSS-Protection`: XSS protection
- `Referrer-Policy`: Controls referrer information
- `Strict-Transport-Security`: HTTPS enforcement (production only)
- `Permissions-Policy`: Restricts browser features

### 4. Authentication Enhancements
- **No auto-login**: Development backdoor removed
- **Secure password hashing**: bcrypt with appropriate cost factor
- **Token validation**: Proper JWT validation with expiration
- **Role-based access**: Admin, Farmer, Processor roles

## 🚀 Deployment Steps

### 1. Environment Setup
```bash
# Set secure JWT secret
export JWT_SECRET=$(openssl rand -base64 32)

# Configure allowed origins
export ALLOWED_ORIGINS="https://yourdomain.com"

# Set appropriate log level
export PROVCHAIN_LOG_LEVEL="warn"
```

### 2. Create First Admin User
```bash
export PROVCHAIN_BOOTSTRAP_TOKEN=$(openssl rand -base64 32)

curl -X POST http://localhost:8080/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{
    "username":"admin",
    "password":"YourSecurePassword123!",
    "bootstrap_token":"'"$PROVCHAIN_BOOTSTRAP_TOKEN"'"
  }'
```

### 3. Start the Server
```bash
# Start with security features enabled
cargo run -- web-server --port 8080

# Or in release mode for production
cargo run --release -- web-server --port 8080
```

### 4. Admin User Management (Post-Bootstrap)
```bash
# Login as admin to get token
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"YourSecurePassword123!"}'

# List users
curl -X GET http://localhost:8080/api/admin/users \
  -H "Authorization: Bearer <admin-token>"

# Create user
curl -X POST http://localhost:8080/api/admin/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{"username":"processor1","password":"ProcessorPass123!","role":"processor"}'

# Delete user
curl -X DELETE http://localhost:8080/api/admin/users/processor1 \
  -H "Authorization: Bearer <admin-token>"

# Filter/paginate users
curl -X GET "http://localhost:8080/api/admin/users?page=1&limit=25&role=processor&q=proc" \
  -H "Authorization: Bearer <admin-token>"

# Rotate user password
curl -X PUT http://localhost:8080/api/admin/users/processor1/password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{"new_password":"ProcessorNewPass456!"}'
```

Admin actions are written to an audit log file.
Default path: `./data/admin_audit.log`
Override path: `PROVCHAIN_AUDIT_LOG_PATH=/secure/path/admin-audit.log`

### 5. Verify Security Setup
```bash
# Check health endpoint for security status
curl http://localhost:8080/health

# Expected response includes:
{
  "status": "healthy",
  "security": {
    "jwt_secret_configured": true,
    "rate_limiting_enabled": true,
    "security_headers_enabled": true,
    "environment": "production"
  }
}
```

## 🔒 Security Best Practices

### 1. JWT Secret Management
- Use 32+ character cryptographically secure secrets
- Rotate secrets periodically (requires system restart)
- Store secrets in secure environment (AWS Secrets Manager, etc.)
- Never log or expose secrets

### 2. User Management
- Enforce strong password policies
- Implement account lockout after failed attempts (rate limiting helps)
- Require password changes for default accounts
- Use least-privilege role assignments

### 3. Network Security
- Deploy behind reverse proxy (nginx, Apache)
- Enable HTTPS with valid SSL certificates
- Configure firewall rules
- Use VPN for administrative access

### 4. Monitoring and Logging
- Monitor authentication failures
- Log rate limiting violations
- Track user creation and role changes
- Set up alerts for suspicious activity

## 🧪 Testing Security Features

### 1. Authentication Security
```bash
# Test rate limiting (should fail after 5 attempts)
for i in {1..7}; do
  curl -X POST http://localhost:8080/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"admin","password":"wrong"}' \
    -v
  echo "Attempt $i"
done
```

### 2. Input Validation Testing
```bash
# Test malicious input in username
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"<script>alert(1)</script>","password":"test"}' \
  -v

# Test SPARQL injection protection
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"query":"DELETE WHERE { ?s ?p ?o }"}' \
  -v
```

### 3. Security Headers Verification
```bash
# Verify security headers are present
curl -I http://localhost:8080/health

# Look for headers:
# Content-Security-Policy
# X-Content-Type-Options
# X-Frame-Options
# X-XSS-Protection
# Referrer-Policy
```

## 🚨 Emergency Procedures

### 1. Compromise Response
1. Immediately change JWT secret: `export JWT_SECRET=$(openssl rand -base64 32)`
2. Restart the server
3. Review and rotate user passwords
4. Check audit logs for unauthorized access
5. Monitor for suspicious activity

### 2. Account Recovery
If locked out due to rate limiting:
1. Wait 5 minutes for rate limit reset
2. Use different IP address
3. Restart server to clear rate limiting state (emergency only)

## 📋 Security Checklist

Before production deployment:

- [ ] JWT secret is set and 32+ characters
- [ ] First admin bootstrapped via `/auth/bootstrap`
- [ ] `PROVCHAIN_BOOTSTRAP_TOKEN` rotated/removed after bootstrap
- [ ] HTTPS is configured with valid certificate
- [ ] CORS origins are restricted to your domains
- [ ] Admin account is created with strong password
- [ ] Rate limiting is enabled
- [ ] Security headers are present
- [ ] Log level is appropriate (warn/error)
- [ ] Firewall rules are configured
- [ ] Monitoring is set up for authentication events
- [ ] Backup procedures are documented
- [ ] Incident response plan is ready

## 🔐 Support and Issues

For security-related issues:
1. Check the application logs: `RUST_LOG=debug cargo run -- web-server`
2. Verify environment variables are set: `env | grep JWT_SECRET`
3. Test with curl commands provided above
4. Review this documentation for configuration steps

For security vulnerabilities or concerns, follow your organization's security disclosure policy.
