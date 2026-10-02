# ADR-0004: Task Queue and Background Execution Strategy

## Context
The legacy backend deployed Celery with Redis for background jobs (backtesting, tournaments, ELO calculations). Celery imposed heavy Python dependencies, synchronous blocking workarounds (`asyncio.run()` within worker tasks), and brittle serialization. In the Rust rewrite, trading engines are removed, and the core workload shifts to real-time transaction event evaluation, audit logging, and behavioral feature aggregation.

## Decision
1. **Phases 1–9 (Initial Infrastructure & Engine)**: Use native Tokio asynchronous task spawning (`tokio::spawn`) with bounded `mpsc` channels and semaphore-based concurrency control for all in-process background operations (e.g. async audit event flush, metrics calculation).
2. **Phase 10 (Distributed Workers & Durable Queuing)**: Formally evaluate **NATS JetStream** for durable, high-throughput distributed event streaming and worker task distribution. NATS provides low-latency messaging, built-in persistence, lightweight operational footprint, and first-class Rust async client support (`async-nats`).

## Alternatives Considered
1. **Retain Celery / Redis Workers**: Rejected because Python workers are strictly forbidden in the production path per [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md).
2. **RabbitMQ / AMQP**: Rejected due to operational heavyweight footprint (Erlang VM, complex cluster management) not justified by current throughput targets.
3. **Apache Kafka**: Rejected as excessive operational overhead for initial deployment tiers; can be re-evaluated if multi-datacenter stream partitioning becomes a hard requirement.
4. **Redis Streams**: Viable secondary option, but lacks built-in consumer deduplication and subject-based filtering capabilities provided natively by NATS JetStream.

## Consequences
- **Positive**: Eliminates external worker infrastructure dependencies during early phases; zero-cost async concurrency in Tokio; clear upgrade path to NATS JetStream in Phase 10 without rewriting domain interfaces.
- **Negative / Risks**: In-process Tokio tasks do not survive unexpected process termination until durable queuing is introduced in Phase 10; requires bounded queues and shutdown hooks to drain in-flight jobs.

## Test Strategy
- Integration tests verifying task completion, cancellation propagation, and bounded buffer backpressure under load.
- Fault-injection tests verifying graceful task draining on `SIGTERM` signals.

## Migration Impact
- Legacy Celery tasks (backtesting, ELO) are designated for removal per the Phase 1 audit component disposition.
- No legacy Celery workers will be ported directly; background audit logging will be handled natively by `edge-workers`.
