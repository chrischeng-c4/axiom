# Tape product requirements

Tape is Axiom's self-hosted stand-in for Google Cloud Pub/Sub. This directory
is the product requirements document: what tape promises to publishers,
subscribers, and operators, written down before the work items that deliver
it. Epics are carved from these sections, not the other way round.

## How this directory is organised

- One file per capability area, named for the area and never for a work item.
  Each `## <title>` section is one promise.
- A shipped promise names the [STATUS](../../STATUS.md) rows that measure it.
  A future promise names the [ROADMAP](../../ROADMAP.md) outcome that owns it
  and ends with `Tracking: not assigned` until its epic exists.
- A future section is written before its epic. When the epic is opened with
  `/aw-grill-me-to-epic`, the epic title is the section title and the section
  heading gains ` (#<iid>)`. The epic's `## Requirements` are carved from the
  section's Promise, so nothing is promised here that an epic cannot measure.
- Every section carries the five parts `/aw-grill-me-to-prd` interviews
  for — Problem, Who, Promise, Non-goals, Neighbours — plus `Open:` lines for
  decisions the epic grill still has to settle. An `Open:` line is a question,
  not a default; the epic body answers it or the human does.
- A new capability area is a change to this index first and to the README
  `### Capability index` when the area ships.

## Positioning

Tape offers Cloud Pub/Sub's feature contract through its own HTTP/1.1 + h2c
API described by `/openapi.json`. The comparison baseline is the Pub/Sub
feature checklist, one row per surface in STATUS. Delivery is at least once;
consumers deduplicate on the offset every message carries.

Boundaries that every section inherits:

- `relay` is the single-queue work-queue broker: one job, one worker. Tape is
  the topic with N independent subscriptions that each see every message.
- `defer` is the Cloud Tasks stand-in. Tape's push delivery is tape's own
  outbound HTTPS, never a hand-off to defer.
- No `pubsub.googleapis.com` wire compatibility, no gRPC, no streaming pull,
  no exactly-once delivery, no export subscriptions, no performance claim
  against another broker.

## Who tape is for

| Reader | What they hold tape to |
|---|---|
| Publisher | One durable write per message, an offset in the answer, refusal before the write when the message is oversized or malformed. |
| Subscriber | Every message on the topic, independently of other subscriptions; a lease per message once acknowledgement ships; push when asked. |
| Operator | A replicated group on Kubernetes with backup, restore, probes, metrics, traces, grants, and runbooks that match the shipped binary. |

## Horizons

| Horizon | Outcome | Section |
|---|---|---|
| H1 | `subscription-ack-and-competing-subscribers` | [subscriptions.md](subscriptions.md) § Subscription ack and competing subscribers |
| H1 | `push-subscriptions` | [subscriptions.md](subscriptions.md) § Push subscriptions |
| H1 | `seek-snapshot-and-retention` | [retention-seek-and-snapshots.md](retention-seek-and-snapshots.md) § Seek, snapshot, and retention |
| H1 | `ordering-keys` | [subscriptions.md](subscriptions.md) § Ordering keys |
| H1 | `attributes-and-filters` | [subscriptions.md](subscriptions.md) § Attributes and filters |
| H1 | `deterministic-failover` | [replication-and-availability.md](replication-and-availability.md) § Deterministic failover |
| H1 | `pubsub-rebaseline` | [api-and-clients.md](api-and-clients.md) § Pub/Sub rebaseline |
| H2 | `schema-validation` | [topics-and-publishing.md](topics-and-publishing.md) § Schema validation |
| H2 | `resource-lifecycle-parity` | [topics-and-publishing.md](topics-and-publishing.md) § Resource lifecycle parity |
| H2 | `live-replica-membership` | [replication-and-availability.md](replication-and-availability.md) § Live replica membership |
| H2 | `multi-shard-topology` | [replication-and-availability.md](replication-and-availability.md) § Multi-shard topology |
| H2 | `quotas-and-scale-transition` | [operations.md](operations.md) § Quotas and scale transition |

H1 is ordered by dependency, not by value: `deterministic-failover` and
`pubsub-rebaseline` first, because the subscription epic has to prove its
lease table across a failover it can observe, and every other epic edits
identity strings the rebaseline regenerates. Then the subscription epic, then
push, seek, ordering, and filters, which all build on the lease table.

## Section index

| Section | File | Kind | Owner |
|---|---|---|---|
| Durable single publish | topics-and-publishing.md | shipped, limited | STATUS `publish-single`, `durable-write-path`, `storage-full-degraded-mode`, `message-attributes`, `ordering-key-on-publish` |
| Resource lifecycle parity | topics-and-publishing.md | outcome | ROADMAP `resource-lifecycle-parity` |
| Schema validation | topics-and-publishing.md | outcome | ROADMAP `schema-validation` |
| Named pull subscriptions | subscriptions.md | shipped, limited | STATUS `subscription-lifecycle-pull`, `pull-sync`, `ack`, `declarative-subscription-provisioning`, `per-topic-authorization`, `delivery-metrics` |
| Subscription ack and competing subscribers | subscriptions.md | outcome | ROADMAP `subscription-ack-and-competing-subscribers` |
| Push subscriptions | subscriptions.md | outcome | ROADMAP `push-subscriptions` |
| Ordering keys | subscriptions.md | outcome | ROADMAP `ordering-keys` |
| Attributes and filters | subscriptions.md | outcome | ROADMAP `attributes-and-filters` |
| Topic retention | retention-seek-and-snapshots.md | shipped, limited | STATUS `message-retention-duration`, `retain-acked-messages`, `seek-to-offset-or-timestamp` |
| Legacy replay and checkpoint routes | retention-seek-and-snapshots.md | shipped, leaving | STATUS `legacy-replay-and-checkpoint-routes` |
| Seek, snapshot, and retention | retention-seek-and-snapshots.md | outcome | ROADMAP `seek-snapshot-and-retention` |
| Replicated group with peer mTLS | replication-and-availability.md | shipped, limited | STATUS `raft-replication`, `raft-peer-mtls`, `leader-failover` |
| Deterministic failover | replication-and-availability.md | outcome | ROADMAP `deterministic-failover` |
| Live replica membership | replication-and-availability.md | outcome | ROADMAP `live-replica-membership` |
| Multi-shard topology | replication-and-availability.md | outcome | ROADMAP `multi-shard-topology` |
| Whole-journal backup and cold seed | operations.md | shipped, limited | STATUS `backup-to-sink`, `cold-seed-bootstrap`, `management-audit` |
| Grants and bounded admission | operations.md | shipped, limited | STATUS `per-topic-authorization`, `flow-control-quotas` |
| Kubernetes operator and direct install | operations.md | shipped, limited | STATUS `k8s-deployment-assets`, `k8s-operator`, `kind-cluster-acceptance`, `gke-zonal-acceptance` |
| Health, metrics, traces, and drain | operations.md | shipped | STATUS `standard-operational-endpoints`, `otlp-tracing`, `bounded-stability-run` |
| Local performance ceiling | operations.md | shipped | STATUS `local-performance-ceiling` |
| Quotas and scale transition | operations.md | outcome | ROADMAP `quotas-and-scale-transition` |
| One discoverable HTTP contract | api-and-clients.md | shipped | STATUS `generated-clients` |
| Pub/Sub rebaseline | api-and-clients.md | outcome | ROADMAP `pubsub-rebaseline` |

Non-goals are not sections. Each file ends with the non-goals that a reader of
that area would otherwise assume, pointing at the ROADMAP entry that gives the
reason: `streaming-pull`, `exactly-once-delivery`, `export-subscriptions`,
`peer-broker-benchmarks`, `pubsub-wire-compatibility`.
