---
id: tape-raft-host-primary-replicas
summary: >
  Wire apps/tape onto libs/raft-host for the primary_replicas trait.
  TapeStateMachine implements raft_host::RaftStateMachine over the shared
  Arc<Mutex<TapeJournal>>: the replicated commands are TapeCommand::Append
  { topic, key, payload, timestamp_ms } and TapeCommand::CheckpointPut
  { topic, consumer, offset, updated_at_ms } (both resolved deterministically
  at the proposing handler, never inside apply); apply calls the existing
  validated TapeJournal::append / put_checkpoint_at methods unchanged
  (append-ordering, retention, stale-checkpoint semantics untouched);
  applied_index survives restart via a small fsynced applied-<node>.idx
  marker in the raft data dir (relay #1207's proven durable-marker recipe,
  not keep's derive-at-recovery); snapshot/restore serialize the WHOLE
  TapeJournal (topics + checkpoints) tagged with the applied index -- valid
  because tape's journal is pure-append with no trimming, unlike relay's
  live/un-acked subset. Auto-mode: bare `tape serve` stays direct-journal;
  when raft_host::cluster::replica_mode() (REPLICAS_PER_SHARD > 1) the serve
  path builds ClusterTopology::from_env("tape", TAPE_PEER_SERVICE, serve
  port, TAPE_PEERS), spawns one RaftHost with TapeStateMachine, mounts
  raft.router() on the serve port OUTSIDE the bearer-auth /topics data
  plane, and routes append/checkpoint-put through raft.propose (leader
  redirect/forward handled by the host). replay/checkpoint-get stay
  node-local reads. Peer TLS is config-surface + fail-fast validation only
  (raft-host's h2c transport has no TLS seam yet -- the shared gap also
  filed against relay/keep/lumen). Verified by a live 3-node kill -9
  failover test proving no committed event loss.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-raft-host-flow
entry: boot
nodes:
  boot:
    kind: start
    label: "serve_main: existing config resolution unchanged (bind, store, grace, auth). New: --data-dir (TAPE_DATA_DIR) for durable raft state and --peer-service (TAPE_PEER_SERVICE, default tape) for the headless-Service peer DNS template"
  mode:
    kind: decision
    label: "raft_host::cluster::replica_mode(): REPLICAS_PER_SHARD > 1 from the standard downward-API quartet? Unset env or 1 replica = single-node -- the auto-mode switch; no tape-specific flags"
  single:
    kind: process
    label: "Single-node path (default, AC1): today's direct-journal router exactly -- append/replay/checkpoint hit Arc<Mutex<TapeJournal>> in-process, --store file persistence unchanged; no raft task, no peer surface, zero behavior change"
  peertls:
    kind: process
    label: "Peer TLS material (config-surface + fail-fast validation ONLY, mirroring relay #1209): tape::peer_tls::PeerTlsConfig::from_env() (TAPE_PEER_TLS_CERT/KEY/CA, TAPE_PEER_MTLS=on|off) loads + validates BEFORE the raft group spawns -- partial config or a mis-pointed path is a startup error. raft-host's h2c peer transport has no TLS acceptor/connector seam yet (the shared gap also filed against relay/keep/lumen), so termination is NOT applied; peer RPCs stay plain h2c either way"
  topo:
    kind: process
    label: "HA path: ClusterTopology::from_env('tape', peer_service, port-from-bind, 'TAPE_PEERS') -- node_id = replica index, voters 0..VOTER_COUNT, peer URLs tape-<ordinal>.<svc>:<port> or the TAPE_PEERS local override. NO local ordinal math"
  spawn:
    kind: process
    label: "TapeRaft::spawn: raft_host::RaftStore::open({data_dir}/raft, node_id, FsyncPolicy::Always); TapeStateMachine::new(Arc<Mutex<TapeJournal>>, marker path) recovers its applied floor from applied-<node_id>.idx; RaftHost::spawn(node_id, membership, peers, store, sm, HostConfig{ snapshot: EveryEntries(SNAPSHOT_EVERY) })"
  mount:
    kind: process
    label: "app = router(state-with-raft).merge(raft.router()): /raft/request-vote, /raft/append-entries, /raft/install-snapshot, /raft/propose, /raftz ride the SAME serve port but OUTSIDE the bearer-auth /topics data plane (cluster traffic, tokenless like probes)"
  write:
    kind: decision
    label: "Data-plane append / checkpoint-put: HA mode? Resolve the deterministic fields BEFORE proposing (timestamp_ms / updated_at_ms via now_ms() at the handler, never inside apply) so every replica computes the identical event/checkpoint"
  propose:
    kind: process
    label: "Encode TapeCommand::Append{topic,key,payload,timestamp_ms} or TapeCommand::CheckpointPut{topic,consumer,offset,updated_at_ms} and raft.propose(cmd) -- the host proposes locally on the leader or forwards to the leader (421 not-leader hint for a direct follower POST), waits for THIS node's apply (read-your-write)"
  apply:
    kind: process
    label: "TapeStateMachine::apply(index, cmd) on EVERY node, once, in index order: locks the shared TapeJournal and calls journal.append(..., Some(timestamp_ms)) or journal.put_checkpoint_at(..., updated_at_ms) -- the SAME validated domain methods the single-node path uses, unchanged append-ordering/retention/stale-checkpoint semantics; the outcome (TapeEvent or Result<ConsumerCheckpoint,TapeError>) is stashed in an OutcomeWindow keyed by raft index so the proposing handler returns the real domain result; then the applied marker file (tmp+rename+fsync in the raft dir) records the floor"
  floor:
    kind: process
    label: "Restart honesty: RaftHost cold-replays resident committed entries; entries at or below the recovered marker are SKIPPED. Tape's journal is pure-append (no delete-on-ack), so a naive replay is merely wasteful without a floor, but checkpoint-put replay could regress or reject a later checkpoint out of order -- the durable marker (relay's proven approach, NOT keep's derive-at-recovery) is the floor for both command kinds so cold replay can never re-apply an already-applied entry"
  snap:
    kind: process
    label: "SnapshotPolicy::EveryEntries: the host calls sm.snapshot() = serde_json of { up_to: applied index, journal: the whole TapeJournal (topics + checkpoints) } -- a full-state snapshot is correct (not a live/un-acked subset like relay) because tape never trims history; node.compact bounds the raft log. restore() (follower InstallSnapshot / cold-start) replaces the shared TapeJournal wholesale and sets the applied floor to up_to"
  local:
    kind: terminal
    label: "replay / checkpoint_get stay NODE-LOCAL reads against the same shared Arc<Mutex<TapeJournal>> the state machine mutates -- no raft round-trip needed for reads (linearizable-enough for this slice; strict read-index consistency is a documented follow-up, same as every other raft_core adopter's default read path)"
  serve:
    kind: terminal
    label: "Auto-mode CLI contract: TAPE_DATA_DIR required in replica/HA mode (fail fast otherwise); TAPE_PEER_SERVICE / TAPE_PEERS as above; TAPE_PEER_TLS_* / TAPE_PEER_MTLS validated at startup. No k8s manifest exists yet for tape (unlike relay) so this slice stops at the CLI + library layer -- a future WI adds the StatefulSet"
edges:
  - { from: boot, to: mode }
  - { from: mode, to: single, label: "no cluster env / 1 replica" }
  - { from: mode, to: peertls, label: "REPLICAS_PER_SHARD > 1" }
  - { from: peertls, to: topo }
  - { from: topo, to: spawn }
  - { from: spawn, to: mount }
  - { from: mount, to: write }
  - { from: write, to: propose, label: "HA mode" }
  - { from: propose, to: apply, label: "propose -> commit -> sole applier" }
  - { from: apply, to: floor, label: "marker persisted per apply" }
  - { from: apply, to: snap, label: "applied - snapshot_index >= threshold" }
  - { from: write, to: local, label: "non-write verbs" }
  - { from: snap, to: serve }
```
