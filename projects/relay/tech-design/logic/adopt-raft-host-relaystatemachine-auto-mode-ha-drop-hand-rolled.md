---
id: relay-adopt-raft-host
summary: >
  Adopt libs/raft-host as relay's consensus driver and delete the hand-rolled
  raft glue (raft_driver/raft_store/raft_config + the relay-raft bin).
  RelayStateMachine implements raft_host::RaftStateMachine over Arc<Relay>:
  the replicated command is PubCommand { subject, message_id, payload,
  headers, priority, not_before } (multi-subject), apply publishes
  idempotently through the engine, applied_index survives restart via a small
  fsynced marker in the raft data dir (the honest floor delete-on-ack +
  the bounded dedupe window require so cold-replay cannot resurrect acked
  work), and snapshot/restore serialize the live (un-acked) engine state via
  new minimal Relay::dump_live/load_live accessors. Auto-mode: bare `relay`
  serve stays direct-engine; when raft_host::cluster::replica_mode()
  (REPLICAS_PER_SHARD > 1) the serve path builds ClusterTopology::from_env
  ("relay", RELAY_PEER_SERVICE, serve port, RELAY_PEERS), spawns one RaftHost
  with RelayStateMachine, mounts host.router() on the serve port OUTSIDE the
  bearer-auth data plane, and routes publish/publish-batch through
  host.propose (redirect/forward handled by the host). Leases/acks/consume
  stay node-local (not replicated — same as the old driver): failover
  redelivery is at-least-once.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-adopt-raft-host-flow
entry: boot
nodes:
  boot:
    kind: start
    label: "serve_main (bare `relay`): existing config resolution unchanged (bind, data dir, auth, reconciler). New: derive the peer port from the bind address and read --peer-service (RELAY_PEER_SERVICE, default relay) for the headless-Service DNS template"
  mode:
    kind: decision
    label: "raft_host::cluster::replica_mode(): REPLICAS_PER_SHARD > 1 from the standard downward-API quartet? Unset env or 1 replica = single-node — the auto-mode switch; no relay-specific flags"
  single:
    kind: process
    label: "Single-node path (default, AC1): today's direct-engine router exactly — publish/publish-batch/lease/ack/consume hit Arc<Relay> in-process; no raft task, no peer surface, zero behavior change"
  topo:
    kind: process
    label: "HA path: ClusterTopology::from_env('relay', peer_service, peer_port, 'RELAY_PEERS') — node_id = replica index, voters 0..VOTER_COUNT, peer URLs relay-<ordinal>.<svc>:<port> or the RELAY_PEERS local override. NO local ordinal math (deletes raft_config.rs; CONTRIBUTING: never re-derive it)"
  spawn:
    kind: process
    label: "RelayRaft::spawn: raft_host::RaftStore::open({data_dir}/raft, node_id, Always) replaces relay's raft_store.rs; RelayStateMachine::new(Arc<Relay>, marker path) recovers its applied floor; RaftHost::spawn(node_id, membership, peers, store, sm, HostConfig with snapshot EveryEntries(1024)) replaces the whole hand-rolled raft_driver.rs tick/pump/flush/persist-before-flush loop"
  mount:
    kind: process
    label: "app = router(state-with-raft).merge(host.router()): /raft/request-vote, /raft/append-entries, /raft/install-snapshot, /raft/publish, /raftz ride the SAME serve port but OUTSIDE the bearer-auth data plane (cluster traffic, tokenless like probes; mTLS is a later slice). The relay-raft bin and its bespoke env contract are deleted"
  pub:
    kind: decision
    label: "Data-plane publish/publish-batch: HA mode? Encode PubCommand with subject, message_id, payload, headers, priority, not_before (multi-subject — upgrade over the old single-subject driver) and host.propose(cmd) — the host proposes locally on the leader or forwards to the leader's /raft/publish (421 not-leader hint for direct peers), waits for THIS node's apply (read-your-write)"
  apply:
    kind: process
    label: "RelayStateMachine::apply(index, cmd) on EVERY node, once, in index order: relay.publish_at(subject, message_id, payload, headers, not_before, priority, now) — idempotent per message_id; outcome {seq, deduped} stashed in an OutcomeWindow keyed by raft index so the proposing handler returns the engine outcome; then the applied marker file (tmp+rename+fsync in the raft dir) records the floor. Engine seqs are node-local; the raft index is the global order"
  floor:
    kind: process
    label: "Restart honesty: RaftHost cold-replays resident committed entries; entries at or below the recovered marker are SKIPPED. Without the floor, delete-on-ack (segments dropped) + the bounded dedupe window would let old committed publishes re-append (resurrect acked work). With FsyncPolicy::Always the engine append is durable before the marker advances, so the floor is never ahead of engine state"
  snap:
    kind: process
    label: "SnapshotPolicy::EveryEntries: the host calls sm.snapshot() = up_to (applied) + relay.dump_live() — for every open (subject, shard) the committed-watermark cut: only un-acked entries (log.range(watermark)); node.compact bounds the raft log. restore() (follower InstallSnapshot / cold-start) = relay.load_live: re-publish each entry idempotently via publish_at (dedupe merges; delayed entries keep not_before), then applied floor := up_to"
  local:
    kind: terminal
    label: "Lease/ack/heartbeat/consume/len stay NODE-LOCAL in HA mode (unchanged handlers): leases are not replicated — exactly the old driver's model, which replicated publishes only. Documented limitation (TD + llm operations topic): on failover the new leader redelivers work that was leased-but-unacked (and acked work not yet trimmed by a snapshot install) — at-least-once, fenced per-node by lease epochs"
  k8s:
    kind: terminal
    label: "k8s/statefulset.yaml switches to the standard contract raft-host reads: POD_NAME (downward API), SHARD_COUNT=1, REPLICAS_PER_SHARD, VOTER_COUNT + RELAY_PEER_SERVICE=relay (headless Service); the image runs the single `relay` entrypoint (Dockerfile drops relay-raft); readiness moves to /readyz"
edges:
  - { from: boot, to: mode }
  - { from: mode, to: single, label: "no cluster env / 1 replica" }
  - { from: mode, to: topo, label: "REPLICAS_PER_SHARD > 1" }
  - { from: topo, to: spawn }
  - { from: spawn, to: mount }
  - { from: mount, to: pub }
  - { from: pub, to: apply, label: "propose -> commit -> sole applier" }
  - { from: apply, to: floor, label: "marker persisted per apply" }
  - { from: apply, to: snap, label: "applied - snapshot_index >= threshold" }
  - { from: pub, to: local, label: "non-publish verbs" }
  - { from: snap, to: k8s }
  - { from: floor, to: k8s }
---
flowchart TD
    boot([serve_main: bind and peer-service resolved]) --> mode{replica_mode: REPLICAS_PER_SHARD gt 1?}
    mode -->|no| single[direct-engine path unchanged — zero flags, zero behavior change]
    mode -->|yes| topo[ClusterTopology::from_env relay / RELAY_PEER_SERVICE / RELAY_PEERS — no local ordinal math]
    topo --> spawn[RelayRaft::spawn: raft_host RaftStore + RelayStateMachine + RaftHost]
    spawn --> mount[merge host.router onto serve port outside bearer auth]
    mount --> pub{publish or publish-batch?}
    pub -->|yes| apply[PubCommand -> host.propose -> apply on every node: idempotent publish_at + OutcomeWindow + fsynced applied marker]
    pub -->|no| local[lease/ack/consume stay node-local — leases not replicated, at-least-once failover]
    apply --> floor[restart: cold-replay skips entries at or below the recovered marker — no resurrection of acked work]
    apply --> snap[snapshot = live dump_live cut; restore = idempotent load_live; compaction bounds the raft log]
    snap --> k8s([statefulset: POD_NAME / SHARD_COUNT / REPLICAS_PER_SHARD / VOTER_COUNT; single relay image])
    floor --> k8s
```
