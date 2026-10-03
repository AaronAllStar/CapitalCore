# EdgeArena Operational Runbook

## 1. System Overview & Architecture Topology

EdgeArena is a high-throughput, low-latency financial decision and intelligence engine rewritten in pure Rust.

```
                           +----------------------+
                           |   Client / Gateway   |
                           +----------+-----------+
                                      | (HTTPS / Bearer JWT)
                                      v
                           +----------------------+
                           |   edge-api (Axum)    |
                           +----------+-----------+
                                      |
         +----------------------------+----------------------------+
         |                                                         |
         v                                                         v
+-----------------------+                                 +-----------------------+
|  Ingestion Pipeline   |                                 |  Health & Metrics     |
|  - Dedup (24h cache)  |                                 |  - /health/liveness   |
|  - Normalization      |                                 |  - /health/readiness  |
|  - Event Validation   |                                 |  - /metrics           |
+----------+------------+                                 +-----------------------+
           |
           v
+-----------------------+       +-------------------------+
|    Decision Service   | ----> | ML Inference (edge-ml)  |
|  - Sliding Windows    |       | - Decision Tree Engine  |
|  - AST Rule Engine    |       | - Integer Basis Points  |
|  - MostRestrictive Arb|       +-------------------------+
+----------+------------+
           |
           v
+-------------------------------------------------+
|     Append-Only Audit Log (edge-audit)          |
|  - Monotonic Sequenced SHA-256 Hash Chaining    |
|  - Tamper-Evident Forensic Verification         |
+-------------------------------------------------+
```

---

## 2. Health & Monitoring Endpoints

| Endpoint | Method | Success Response | Description |
|---|---|---|---|
| `/health/liveness` | `GET` | `{"status":"HEALTHY","healthy":true}` | Verifies process liveness. Used by Kubernetes/Docker liveness probes. |
| `/health/readiness` | `GET` | `{"status":"HEALTHY","healthy":true}` | Evaluates dependencies (PostgreSQL, EventBus, Deduplicator). |
| `/metrics` | `GET` | JSON snapshot of counters/gauges | Real-time metrics: request count, decision distributions, latencies. |
| `/openapi.json` | `GET` | OpenAPI 3.0 specification | Live API contract documentation. |

---

## 3. Service Level Objectives (SLOs) & Thresholds

| Metric | Target | Warning Threshold | Critical Alert |
|---|---|---|---|
| **Decision Latency (p50)** | < 10 µs | > 50 µs | > 200 µs |
| **Decision Latency (p99)** | < 5 ms | > 15 ms | > 50 ms |
| **Availability** | 99.99% | < 99.95% | < 99.90% |
| **Throughput Baseline** | > 1,000 TPS | < 500 TPS | < 100 TPS |
| **Duplicate Rejection Accuracy** | 100.0% | < 100% | Any duplicate leak |
| **Audit Chain Integrity** | 100.0% | Any hash mismatch | Any broken chain |

---

## 4. Operational Procedures

### 4.1 Deployment (Docker Compose)
To start the production stack:
```bash
docker compose -f docker-compose.prod.yml up -d
```

To view structured JSON logs:
```bash
docker compose -f docker-compose.prod.yml logs -f edge-api
```

To verify health:
```bash
curl -s http://localhost:8080/health/readiness | jq .
```

### 4.2 Graceful Shutdown
EdgeArena handles `SIGTERM` and `SIGINT` (Ctrl+C) gracefully:
1. Stops accepting new inbound HTTP requests.
2. Drains active requests with a 10-second timeout.
3. Flushes in-flight events to the audit log.
4. Exits with return code 0.

### 4.3 Dead-Letter Queue (DLQ) Triage & Replay
When unrecoverable or poison events are encountered:
1. Events are isolated in the `DeadLetterQueue` with original payload, error trace, and attempt counter.
2. Query DLQ messages:
   ```bash
   curl -s -H "Authorization: Bearer <ADMIN_TOKEN>" http://localhost:8080/api/v1/dlq
   ```
3. Inspect `last_error` and `is_poison` flags.
4. If upstream issue resolved, trigger redrive/replay.

### 4.4 Cryptographic Audit Chain Verification
In the event of forensic audits or suspected database corruption:
1. Call `AuditLog::verify_integrity()`.
2. The engine recalculates SHA-256 hashes across monotonic sequence IDs:
   `H_n = SHA256(seq_n || prev_hash || event_payload)`.
3. If any record has been modified, deleted, or inserted out of order, the chain verification returns `false` with the offending sequence ID.

### 4.5 Secret & Key Rotation
- **Ed25519 JWT Signing Keys**: Rotate keys every 90 days. During rotation, configure the authorizer to accept both `N` (new) and `N-1` (previous) public verification keys for a 24-hour grace period before invalidating `N-1`.
- **Database Credentials**: Use dynamic PostgreSQL secret rotation via IAM or Vault.
