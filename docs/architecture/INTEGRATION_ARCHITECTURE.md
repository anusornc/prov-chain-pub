# ProvChainOrg Integration Architecture

**Version:** 1.1
**Last Updated:** 2026-08-31
**Author:** Anusorn Chaikaew (Student Code: 640551018)

---

## Claim and evidence boundary

This document separates **current foundation** from **accepted target** and **future integration**.
Examples are non-normative unless linked to a current route/test or an accepted ADR. External
ERP/vendor adapters, IoT ingestion, webhooks, and their schemas are future work; they are not current
implementation or thesis evidence. The bounded ProvChain bridge is an accepted target whose
implementation and six-process conformance evidence remain pending. PBFT, heterogeneous/SPV bridge,
operational deployment, and production-pilot controls are also future work.

## 1. External System Integrations

### 1.1 Future ERP System Integration

**Status:** Future integration target. No ERP-vendor connector is claimed by the current repository.

**Purpose:** Transform an authenticated enterprise event into a bounded unsigned RDF request under
the target ledger/profile and ontology package.

The current `/auth/login` and `/api/datasets/import-turtle` routes are reusable HTTP/JWT
foundations, not an ERP contract and not evidence that writes pass the accepted universal Final
Admission/journal path. A future adapter must define and test:

- vendor authentication and credential custody;
- deterministic source-to-package mapping with versioned fixtures;
- request-size, batch, timeout, and retry/idempotency bounds;
- fail-closed propagation of semantic/admission rejection; and
- receipts that distinguish node-local commitment from three-node convergence.

---

### 1.2 Future IoT Sensor Integration

**Status:** Future integration target. `/ws/iot` is not a current route, and the current generic
`/ws` event-subscription route is not sensor ingestion.

**Purpose:** Authenticate a sensor or gateway, bound and normalize readings, map them under a pinned
ontology package, then submit an unsigned ordinary request through universal Final Admission.

WebSocket or MQTT may be evaluated later, but no transport, wire schema, units/ranges, device list,
batching policy, or broker configuration is declared current here. A future adapter must add device
identity/authorization, replay protection, bounded queues/backpressure, unit normalization,
package-conformance fixtures, and crash-safe retry evidence.

---

## 2. Bounded ProvChain-to-ProvChain Bridge

### 2.1 Accepted contract and current boundary

The accepted bridge is [ADR 0037](./ADR/0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md): one-hop, byte-for-byte copying of one complete public `OrdinaryProvenanceV1` payload between two explicitly pinned ProvChain ledger instances using the same state, semantic, and ontology-package contract.

The source must commit a target-bound Bridge Export Declaration and reach all-three Network
Convergence. The target profile must already pin the source ledger, governance root, profile,
signed manifest, three receipt signers, and allowed contracts. The target Scheduled Authority and
every ordinary target Final Admission gate remain mandatory. Only target Ledger Journal append plus
`fsync` creates terminal Imported state, and replay state is reconstructed from that journal.

Current `src/interop/bridge.rs` is a legacy in-process one-signature/RAM-replay prototype. It does
not implement or evidence the accepted contract.

All ordinary, privacy-control, and bridge requests share one PoA Proposal Coordinator for the
pending turn. It serializes and fully preflights unsigned requests and durably fences one exact
unsigned proposal body before the Scheduled Authority signs. Parent advancement may refresh only a
request that remains unselected, has not begun fence persistence, and has not invoked the signer.
Once selection or a fence-write attempt begins, only the exact fenced body and its deterministic
signature may be recovered or rebroadcast, including when the signer response is unknown; a
distinct signed proposal for that turn is Equivocation, not a Final Admission race. The Signing
Fence is safety state, not a second ledger commit authority.

### 2.2 Protocol flow

```mermaid
sequenceDiagram
    autonumber
    participant SA as Source Scheduled Authority
    participant SF as Source Final Admission
    participant SN as Three Source Nodes
    participant R as Untrusted Relayer
    participant BA as Target Bridge Adapter
    participant TC as Target PoA Proposal Coordinator
    participant TA as Target Scheduled Authority
    participant TF as Target Final Admission
    participant TN as Three Target Nodes

    SA->>SF: OrdinaryProvenanceV1 + target-bound export declaration
    SF->>SF: Verify source contracts; append envelope + fsync
    SF->>SN: Replicate exact committed envelope and prefix
    SN-->>R: Exact envelope + 3/3 matching Commit Receipts
    Note over R: Assemble exact Bridge Proof Bundle; derive no authority
    R->>BA: Proof bundle + unchanged public payload
    BA->>BA: Strict decode/bounds + verified Effective Bridge State
    alt Exact transfer already Imported
        BA-->>R: AlreadyImported + stored target reference; no proposal
    else Same transfer ID conflicts
        BA-->>R: ReplayConflict; no proposal
    else Transfer absent and request valid
        BA->>TC: Unsigned exact proof/payload request
        TC->>TC: Serialize/coalesce + complete read-only preflight
        alt Parent advances before selection/fence persistence
            TC->>TC: Refresh parent and rerun complete unsigned preflight
        else Selected for the current PoA Turn
            TC->>TC: Durably fence exact unsigned proposal body and digest
            TC->>TA: Exact fenced canonical proposal body
            TA-->>TC: One signature for the PoA Turn
            TC->>TF: Exact signed target proposal + Bridge Origin Evidence
            TF->>TF: Verify target parent/turn/contracts
            TF->>TF: Verify pinned source trust + 3/3 receipts + derived transfer ID
            TF->>TF: Run target RDF/state/package/full-union SHACL admission
            alt Every gate passes
                TF->>TF: Append complete target envelope + fsync
                TF->>TN: Replicate exact target envelope and prefix
                TN-->>R: Target Commit Receipts as available
            else Any deterministic gate fails
                TF-->>TC: Rejected; no journal or bridge-state mutation
                Note over TC,TF: Safety incident and stalled turn;<br/>no replacement proposal
            else Node incapable or commit outcome unknown
                TF-->>TC: No global verdict
                TC->>TC: Recover journal + Signing Fence; query transfer
                Note over TC,TF: Retry only exact signed bytes if still uncommitted
            end
        end
    end
```

### 2.3 Explicit v1 boundaries

| Included | Excluded |
|---|---|
| Exact public RDF payload | RDF or ontology transformation |
| Native source envelope with no bridge origin | Private payload, grants, keys, or privacy transitions |
| Pinned source profile/manifest and all three receipts | Proof-supplied trust or one configurable authority |
| Shared pre-sign coordinator/fence, target PoA, and full target Final Admission | Competing signed proposals, direct bridge append, or optional SHACL |
| Journal-derived `Imported`, `AlreadyImported`, and `ReplayConflict` behavior | Process-local replay set or exactly-once transport claim |
| One direct ProvChain-to-ProvChain hop | Multihop, heterogeneous, SPV, lock/mint, or asset transfer |

---

## 3. API Reference

This section records **current route foundations**, not a stable external-vendor contract and not
proof of target Final Admission. Integrators must verify request/response models against the running
build and pin an API version before relying on them.

### 3.1 REST API Endpoints

#### Authentication

`POST /auth/login` exists as a current JWT foundation. Credentials, bootstrap policy, token claims,
expiry, TLS termination, and production identity controls are deployment/security concerns and are
not specified by this integration reference.

#### Import Turtle Dataset

```http
POST /api/datasets/import-turtle
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "turtle_data": "@prefix ex: <http://example.org/> . ex:Product ex:name \"Widget\" ."
}
```

The current handler returns development block metadata. Treat it as a foundation response only: it
does not yet prove complete-envelope journal commitment or three-node convergence. The accepted
target response must expose those states distinctly.

#### SPARQL Query

```http
POST /api/sparql/query
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "query": "SELECT ?s ?p ?o WHERE { ?s ex:suppliedBy ?o . ?o ex:locatedIn \"warehouse_a\" }",
  "format": "json"
}
```

The current response model includes results, `execution_time_ms`, and `result_count`. Result shape
depends on query/format. A query response is a read of projected state, not a ledger receipt or a
claim that package-controlled reasoning ran.

### 3.2 WebSocket API

The current web server exposes generic `/ws` event subscription with authenticated connection
handling and messages such as subscribe/unsubscribe, ping/pong, event, error, and connected. This is
separate from the node-to-node exact-envelope protocol and must not be described as P2P consensus,
sync, IoT ingestion, or durable delivery. Its exact JSON schema must be taken from the current Rust
types and pinned by a consumer contract test before external use.

---

## 4. Integration Patterns

### 4.1 Future Webhook Notifications

Webhook delivery is a future integration pattern. The repository has monitoring configuration
structures/logging stubs, but no current production HTTP delivery contract, signing scheme,
durable outbox, retry/dead-letter behavior, or conformance evidence. No example configuration or
payload in this document is a usable current webhook contract.

A future design must trigger only from a committed journal envelope, include a stable event ID and
ledger/profile/prefix reference, authenticate payloads without embedding secrets in configuration,
bound retries, survive restart, and state explicitly that delivery is at-least-once unless stronger
evidence exists.

### 4.2 Polling vs Push

| Pattern | Current boundary | Future requirement |
|---------|------------------|--------------------|
| **Polling** | Current read APIs may be polled; no vendor SLA | Pin query/prefix semantics and rate bounds |
| **Generic `/ws` push** | Current non-durable event-subscription foundation | Pin schema/auth and define reconnect/gap recovery |
| **Webhook** | Not implemented as an external delivery contract | Journal-derived durable outbox, authentication, bounded retries |

---

## 5. Integration Testing

### 5.1 Planned harness requirements

There is no current ERP- or IoT-specific integration harness. Earlier script names were design
placeholders, not repository evidence.

| Harness | Minimum evidence before claiming support |
|---------|------------------------------------------|
| ERP adapter | Real selected vendor sandbox/fixture; authenticated mapping; invalid/boundary cases; restart-safe retry; rejection and node-commit/convergence receipts archived |
| IoT adapter | Real transport and schema; device authentication/replay defense; bounds/backpressure; unit/package fixtures; disconnect/restart behavior archived |
| Generic WebSocket consumer | Current Rust message schema pinned; auth, subscription, reconnect, lag/gap behavior, and non-durability documented and tested |
| Webhook | Committed-envelope trigger; payload authentication; durable outbox; duplicate/retry/dead-letter and restart cases |
| Bounded bridge | Two independent three-process ProvChain networks; exact source/target envelopes, manifests, strict feature graphs, 3/3 receipts, rejection, replay/conflict, crash, and restart artifacts required by ADR 0037 |

Only the bounded bridge harness is inside the locked thesis reference-system scope. ERP, IoT,
generic external notification hardening, and production operational controls remain future work.

---

## 6. Related Documentation

- [Data Flow Architecture](./DATA_FLOW_ARCHITECTURE.md) - Detailed flow diagrams
- [Container Architecture](./CONTAINER_ARCHITECTURE.md) - Container interactions
- [Security Architecture](./SECURITY_ARCHITECTURE.md) - Secure integrations

---

**Contact:** Anusorn Chaikaew (Student Code: 640551018)
**Thesis Advisor:** Associate Professor Dr. Ekkarat Boonchieng
