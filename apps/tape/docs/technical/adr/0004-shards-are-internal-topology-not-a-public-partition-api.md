# ADR 0004 — Shards are internal topology, never a public partition API (#1815)

Status: Accepted (2026-08-26; recorded from the retired `external-contracts/topology/behavior/shard-topology.md`).

## Context

Tape reuses the shared `service-k8s` cluster shape, whose CRD carries
`shardCount` and `replicasPerShard` as independent storage and availability
axes (`src/operator/crd.rs:38-45`). Kafka-style systems expose partitions to
publishers and consumers as a public addressing unit; the question was
whether tape's shards would be that kind of surface.

## Decision

- Tape's external contract is topic → N subscriptions. Shards and replicas
  are storage and availability topology behind that contract, and no route,
  CLI verb, or client model names a shard.
- Today the render pins `shard_count: 1` (`src/operator/render.rs:678-681`);
  `replicasPerShard` is the only scale knob, and `replicasPerShard > 1` is
  what switches `serve` into Raft replica mode.
- When `ROADMAP.md#multi-shard-topology` raises the pin, ordering keys
  (`ROADMAP.md#ordering-keys`) must hold across shards without the client
  learning which shard holds a key.
- Backup snapshots seed cold recovery and empty-PVC bootstrap (ADR 0002);
  live replicas converge through Raft log and InstallSnapshot mechanics.

## Consequences

- A publisher or subscriber written against tape today keeps working when
  the cluster is resharded.
- Any future partition-aware feature has to be expressed in Pub/Sub terms —
  ordering keys, subscription state — rather than as a shard id on the wire.

## Status of work

Landed as a pin; the multi-shard half is a roadmap outcome. Gate:
`cargo test -p tape --features operator --test operator --test operator_render_provision_topics`.
