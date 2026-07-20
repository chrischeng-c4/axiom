# relay

## Brief

`relay` is the online **single-cast pull work-queue broker** in the Axiom stack
(RabbitMQ/SQS-shaped): a producer publishes a task, a worker **pulls** (leases)
it, runs it, and acks — each message is delivered exactly once to one of the
competing consumers, then reclaimed (**delete-on-ack**). It owns work-queue
leasing with lease-expiry redelivery, dead-lettering, priority, short delayed
visibility, h2c/OpenAPI transport, and raft-backed HA. Payloads stay opaque JSON
so higher-level systems (loom, worker runtimes) use relay without relay learning
their domain model.

Relay is the pull side of Axiom's queue family; [`defer`](../defer) is the push
side for scheduled HTTP dispatch. Relay is deliberately **not** a Kafka/pub-sub
platform: durable topic replay/retention is [`tape`](../tape)'s job, and
push/ETA task dispatch is [`defer`](../defer)'s. Concurrency is
**client-driven** — more workers = more throughput; relay has no server-side
rate/concurrency governor (that is `defer`). Tenancy is a deployment concern —
relay is single-tenant per deployment; run one per tenant (k8s namespace). See
the boundary notes in the ecosystem docs.

Priority is shared with `defer` as an unsigned byte: `0` is lowest, `255` is
highest, the default is `10`, and higher values lease first. In relay, priority
orders only entries that are already visible in the work queue; long-horizon ETA,
rate limiting, and HTTP target dispatch remain Defer concerns.

Relay ships as **one binary**: bare `relay` serves (h2c broker; raft-runtime
auto-mode HA — see [`HA.md`](HA.md)), and the same bin carries the offline
`spec`/`spec gen`, `backup`, `k8s crd|operator|instance`, `dockerfile render`,
and shared `llm`/`upgrade`/`issue` verbs (WIs #1204-#1209, the
service-archetype convergence).

## Capability Contract

Machine-readable capability contract for Relay. Full contract:
[CAPABILITIES.md](CAPABILITIES.md).
