# Tape roadmap

## Purpose

This document records future product outcomes and explicit non-goals. It does
not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.
This file keeps stable outcome IDs so current limits can point to one future
destination without copying tracker state.

Tape's peer for feature parity is Google Cloud Pub/Sub. Each near-term outcome
below closes one group of STATUS rows toward that checklist; the later outcomes
follow once subscriptions carry per-message state.

## Near-term outcomes

### Subscription ack and competing subscribers

- ID: `subscription-ack-and-competing-subscribers`
- Outcome: A subscription is a configured resource whose pull hands out
  per-message ackIds under an ack deadline, whose callers ack, extend, or nack
  each message, and whose expired messages are redelivered to any puller on the
  same subscription, with a retry policy and a dead-letter topic after the
  configured number of delivery attempts.
- Boundary: In-flight state and subscription configuration travel through Raft
  as replicated commands, and declarative provisioning proposes through the
  same path instead of writing the journal directly. Grants become
  subscription-scoped. The cumulative cursor stays as the internal position
  behind the lease table. This outcome does not add push, streaming pull, or
  exactly-once delivery.
- Completion evidence: End-to-end cases prove that two pullers on one
  subscription split the messages, that an unacked message returns after its
  deadline, that a nack returns it immediately, that a modify-ack-deadline
  call defers it, that the configured attempt limit forwards it to the
  dead-letter topic, that the lease table survives leader failover, and that
  an oldest-unacked-age metric is exposed. The existing cumulative-cursor
  cases are rewritten against the lease model, not deleted.
- Tracking: Not assigned.

### Push subscriptions

- ID: `push-subscriptions`
- Outcome: A subscription can carry a push configuration with an HTTPS
  endpoint and a bearer token, and tape delivers each message to it, treating
  a 2xx as the ack and anything else as a failed attempt that backs off and
  falls back to the pull-visible in-flight state.
- Boundary: Tape is the outbound HTTP client; nothing is handed to defer. The
  endpoint must be HTTPS. Google-signed OIDC tokens and non-HTTPS targets are
  outside this outcome.
- Completion evidence: An end-to-end case runs a local TLS receiver, proves
  that a 2xx acks the message, that a 5xx redelivers after the configured
  backoff, that a receiver outage leaves the message pullable, and that the
  push token is redacted from logs and metrics.
- Tracking: Not assigned.

### Seek, snapshot, and retention

- ID: `seek-snapshot-and-retention`
- Outcome: A subscription can seek to a timestamp, to an offset, or to a named
  snapshot resource, and topic retention proves its age-based expiry.
- Boundary: This is the outcome in which the legacy replay, streaming replay,
  consumer checkpoint, and backfill surfaces leave the public API, the
  offline `tape replay` and `tape checkpoint` verbs retire, and the retention
  shape narrows to a duration with the offset floor and protected consumers
  becoming internal. The acceptance scripts move their readback onto
  subscription pull first. A decision record settles the naming collision
  between a subscription snapshot and the whole-journal backup.
- Completion evidence: End-to-end cases prove seek to time, seek to offset,
  snapshot create, list, delete, and seek-to-snapshot, retention expiry by
  age with a protected floor, and that every retired route answers 404 while
  the route inventory, the committed OpenAPI snapshot, and the generated
  clients agree.
- Tracking: Not assigned.

### Ordering keys

- ID: `ordering-keys`
- Outcome: Messages that share an ordering key are delivered to their holder
  in publish order, including across redelivery and competing pullers, while
  messages without a key carry no ordering promise.
- Boundary: Shards stay internal topology; ordering is a property of the
  delivery path, not of a public partition API. Ordering across more than one
  shard is deferred to the multi-shard outcome.
- Completion evidence: An end-to-end case publishes interleaved keys from
  several publishers, pulls with several competing callers, forces a
  redelivery, and proves per-key order held while unkeyed messages
  interleaved freely.
- Tracking: Not assigned.

### Attributes and filters

- ID: `attributes-and-filters`
- Outcome: A publish carries a string-to-string attribute map alongside the
  payload, and a subscription stores a filter expression that is evaluated at
  delivery, with filtered messages acked implicitly.
- Boundary: The single `key` becomes the ordering key, not an attribute.
  Filters read attributes only; payload-content and schema-aware filtering
  are outside this outcome.
- Completion evidence: End-to-end cases prove attributes round-trip through
  pull and push, that a filtered subscription receives only matching messages,
  that a filter change applies to messages published after it, and that the
  filtered-out messages do not appear as lag.
- Tracking: Not assigned.

### Deterministic failover

- ID: `deterministic-failover`
- Outcome: The replication and failover cases observe each step through their
  own readiness and leadership surfaces instead of one shared wall-clock
  deadline, and the follower-forwarded append case passes against the shared
  runtime's JSON publish handler.
- Boundary: This is test-harness and runtime-adapter work; it changes no
  public contract. It is the prerequisite for proving that a lease table
  survives failover in the subscription outcome.
- Completion evidence: Every `raft_cluster` and `raft_failover` case is green
  twenty consecutive times with a single test thread on a loaded host, and no
  case carries a shared deadline constant.
- Tracking: [#3926](https://github.com/chrischeng-c4/axiom/issues/3926)

### Pub/Sub rebaseline

- ID: `pubsub-rebaseline`
- Outcome: Every identity string tape emits describes it as the Cloud Pub/Sub
  stand-in, and the operational documents live where the gates expect them.
- Boundary: The crate description, the OpenAPI info block, the served route
  descriptions, the custom resource documentation, and the committed OpenAPI
  snapshot are regenerated together. The cold-restore and disk-full runbooks
  move under a runbooks directory with the alert rule and the operator render
  pointing at the new path, and the deployment handoff page retires. No
  product behaviour changes.
- Completion evidence: The route-parity, generated-client, backup-destination,
  and operator render gates pass against the regenerated snapshot and the new
  runbook path, and the deployment handoff page is no longer tracked.
- Tracking: Not assigned.

## Later outcomes

### Schema validation

- ID: `schema-validation`
- Outcome: A topic can bind a schema in Avro, Protobuf, or JSON Schema form,
  and a publish that does not conform is refused with 400 before it reaches
  the WAL.
- Boundary: Schemas are tape resources with revisions; the validator runs in
  the publish path only. Schema evolution rules beyond revision pinning are
  outside this outcome.
- Completion evidence: End-to-end cases prove create, list, and delete of a
  schema, a conforming publish accepted, a non-conforming publish refused
  without an offset, and a revision change applying to later publishes only.
- Tracking: Not assigned.

### Resource lifecycle parity

- ID: `resource-lifecycle-parity`
- Outcome: Topics have explicit create, delete, and list routes, publish
  accepts a batch, and a subscription can carry an expiration policy or be
  detached from its topic.
- Boundary: Implicit topic creation stays available behind the explicit
  routes so current callers keep working. Any item here that the product does
  not want moves to the non-goals list rather than staying open.
- Completion evidence: End-to-end cases prove each route, that a batch
  reports per-message offsets, that an expired subscription disappears, and
  that a detached subscription stops receiving messages.
- Tracking: Not assigned.

### Live replica membership

- ID: `live-replica-membership`
- Outcome: A replica-count change on the custom resource adds a learner,
  promotes it once caught up, or removes a voter, without restarting the
  surviving members.
- Boundary: The shared runtime already exposes learner and promotion calls;
  this outcome wires the operator to them and proves the transition. Shard
  count is not part of it.
- Completion evidence: An operator case scales one to three and three to one
  while a publisher keeps appending, and proves no committed message is lost
  and no surviving member restarts.
- Tracking: Not assigned.

### Multi-shard topology

- ID: `multi-shard-topology`
- Outcome: A `Tape` custom resource can declare more than one shard, and
  topics are placed across shards while ordering keys keep their promise
  within a key.
- Boundary: Shards remain internal topology and never appear in the public
  API. Cross-shard transactions are outside this outcome.
- Completion evidence: An operator case renders and reconciles a two-shard
  instance, and an end-to-end case proves keyed order across a shard split.
- Tracking: Not assigned.

### Quotas and scale transition

- ID: `quotas-and-scale-transition`
- Outcome: Per-topic and per-subscription quotas are enforced, the default
  router enables bounded admission, and a scale transition is proven on GKE.
- Boundary: Quotas are counts and bytes per window; billing-style accounting
  is outside this outcome. The GKE proof stays zonal.
- Completion evidence: End-to-end cases prove a quota refusal with the shared
  error envelope, and the GCP acceptance script proves a replica-count
  transition under load with no committed loss.
- Tracking: Not assigned.

## Non-goals

### Streaming pull

- ID: `streaming-pull`
- Reason: Tape offers no gRPC and no wire compatibility, so a bidirectional
  streaming pull has no client to serve. Unary pull and push are the two
  delivery paths, and the legacy streaming replay route leaves with the seek
  outcome.

### Exactly-once delivery

- ID: `exactly-once-delivery`
- Reason: Delivery is at least once, and consumers deduplicate on the offset
  that every event carries. An exactly-once promise would require a
  transactional ack path that the product has not asked for.

### Export subscriptions

- ID: `export-subscriptions`
- Reason: Tape does not write messages to object storage or another service
  on a subscription's behalf. The whole-journal backup is disaster recovery,
  not an export path.

### Peer-broker benchmarks

- ID: `peer-broker-benchmarks`
- Reason: Performance is measured only against tape's own release-mode
  baseline. The earlier NATS JetStream and Kafka calibrations are recorded as
  history in the benchmark notes and are not a product claim.

### Pub/Sub wire compatibility

- ID: `pubsub-wire-compatibility`
- Reason: Feature parity is offered through tape's own h2c and OpenAPI API.
  Speaking the Google API surface would tie the product to a protocol it does
  not control.
