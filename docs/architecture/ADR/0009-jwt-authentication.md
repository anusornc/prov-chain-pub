# ADR 0009: Use JWT for API Authentication

**Status:** Accepted
**Date:** 2026-01-28
**Context:** REST API authentication and authorization

---

## Decision

**Use JWT (JSON Web Tokens) for all API authentication in ProvChainOrg.**

### Rationale

**Alternatives Considered:**
| Method | Pros | Cons | Decision |
|--------|------|------|----------|
| **JWT** | • Stateless, scalable<br/>• Standard, widely supported<br/>• No server-side session storage | • Token revocation complex | ✅ **Chosen** |
| **Session-Based** | • Easy revocation | • Server-side state<br/>• Doesn't scale horizontally | ❌ Rejected |
| **API Keys** | • Simple | • No user context<br/>• Hard to rotate | ❌ Rejected |
| **OAuth2** | • Delegation support | • Overkill for single system<br/>• Complex | ❌ Rejected |

### Benefits

1. **Stateless:** No session storage, scales horizontally
2. **Standard:** JWT libraries available in all languages
3. **User Context:** Claims carry user identity and roles
4. **Security:** Ed25519 signatures for token integrity

---

## Implementation

**Crate:** `jsonwebtoken`
**Algorithm:** Ed25519 (not RS256 - faster)
**Claims:** `sub` (user), `role` (admin/user), `exp` (expiration)

---

## Related Decisions

- [ADR 0004](./0004-use-ed25519-signatures.md): Use Ed25519 for Signatures (JWT signature algorithm)

---

**Authors:** Anusorn Chaikaew (anusorn.c@crru.ac.th)
**Approval Date:** 2026-01-28
