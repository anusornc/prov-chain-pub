# ADR 0013: Use Prometheus + Grafana for Monitoring

**Status:** Accepted
**Date:** 2026-01-28
**Context:** System observability and performance monitoring

---

## Decision

**Use Prometheus + Grafana for system monitoring and observability.**

### Rationale

**Alternatives Considered:**
| System | Pros | Cons | Decision |
|--------|------|------|----------|
| **Prometheus + Grafana** | • Cloud-native standard<br/>• Pull-based (no push complexity)<br/>• Rich visualization | • Not push-based | ✅ **Chosen** |
| **InfluxDB** | • Push-based, time-series optimized | • Less standard, expensive | ❌ Rejected |
| **Datadog** | • Full-featured, easy SaaS | • Cost, vendor lock-in | ❌ Rejected |
| **ELK Stack** | • Log aggregation | • Complex, resource-heavy | ❌ Rejected |

### Benefits

1. **Standard:** CNCF project, cloud-native standard
2. **Pull-Based:** No push complexity, service discovery
3. **Grafana:** Rich dashboards, alerting
4. **Ecosystem:** Large community, many integrations

---

## Implementation

**Metrics:** Prometheus 2.x
**Visualization:** Grafana 9.x
**Tracing:** Jaeger (optional)
**Logging:** Structured JSON (tracing crate)

**Key Metrics:**
- `provchain_transactions_total` (Counter)
- `provchain_blocks_created` (Counter)
- `provchain_spq_query_duration` (Histogram)
- `provchain_consensus_duration` (Histogram)
- `provchain_peer_count` (Gauge)

---

**Authors:** Anusorn Chaikaew (anusorn.c@crru.ac.th)
**Approval Date:** 2026-01-28
