---
id: relay-raft-persistence
summary: Persist Raft hard state (term, votedFor, log) so a node survives restart and never double-votes or forgets acked entries. RaftNode stays pure — add a serializable PersistedState + persisted()/from_persisted(); a file-backed RaftStore saves/loads under a data dir honoring FsyncPolicy; the driver persists before flushing the outbox. Standalone, no external dep.
capability_refs:
  - id: competitor-feature-parity
    role: primary
    gap: durable-raft-hard-state-restore
    claim: durable-raft-hard-state-restore
    coverage: full
    rationale: "Defines durable raft hard-state persistence and restart restore behavior for term, vote, and replicated log state."
  - id: long-running-stability
    role: primary
    gap: raft-hard-state-restart-safety
    claim: raft-hard-state-restart-safety
    coverage: full
    rationale: "Defines restart-safe persisted raft term, vote, and log state."
fill_sections: [logic, unit-test, changes]
---

# relay Raft hard-state persistence + FsyncPolicy (crash-safe single voter)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-raft-persistence-flow
entry: mutate
nodes:
  mutate:
    kind: start
    label: "RaftNode mutates hard state (term/votedFor change, log append/truncate, commit)"
  snap:
    kind: process
    label: "driver takes node.persisted() = PersistedState{term, voted_for, log}"
  save:
    kind: process
    label: "RaftStore.save(state): write under data_dir, fsync per FsyncPolicy (hard state is tiny + safety-critical -> always durable)"
  flush:
    kind: decision
    label: "persisted before sending?"
  send:
    kind: terminal
    label: "only now flush the outbox -> no vote/ack leaves before it is durable"
  hold:
    kind: terminal
    label: "save failed -> surface error, do NOT send (stay safe)"
  restart:
    kind: process
    label: "on restart: RaftStore.load() -> RaftNode::from_persisted(id, membership, state)"
  resume:
    kind: terminal
    label: "node resumes as Follower with log intact + remembers votedFor (re-elects without losing committed data)"
edges:
  - { from: mutate, to: snap }
  - { from: snap, to: save }
  - { from: save, to: flush }
  - { from: flush, to: send, label: "ok" }
  - { from: flush, to: hold, label: "io error" }
  - { from: restart, to: resume }
---
flowchart TD
    mutate([hard-state mutation]) --> snap[node.persisted()]
    snap --> save[RaftStore.save + fsync per policy]
    save --> flush{persisted before send?}
    flush -->|ok| send([flush outbox; nothing sent before durable])
    flush -->|io error| hold([surface error, do not send])
    restart[restart: RaftStore.load] --> resume([from_persisted: log intact, votedFor remembered])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-raft-persistence-test-plan
entry: suite
nodes:
  suite: { kind: start, label: "Raft persistence tests" }
  t_round: { kind: process, label: "save a PersistedState{term, votedFor, log} then load from a fresh RaftStore" }
  a_round: { kind: terminal, label: "assert term/votedFor/log round-trip exactly" }
  t_resume: { kind: process, label: "sole voter commits entries; persist; from_persisted into a new node" }
  a_resume: { kind: terminal, label: "assert log intact, can resume, and surviving committed entries are present" }
  t_vote: { kind: process, label: "node grants a vote (votedFor set); persist; restore" }
  a_vote: { kind: terminal, label: "assert restored node remembers votedFor and will not grant a second different candidate in that term" }
  t_empty: { kind: process, label: "load from an empty data dir" }
  a_empty: { kind: terminal, label: "assert None (fresh start, no error)" }
edges:
  - { from: suite, to: t_round }
  - { from: t_round, to: a_round }
  - { from: suite, to: t_resume }
  - { from: t_resume, to: a_resume }
  - { from: suite, to: t_vote }
  - { from: t_vote, to: a_vote }
  - { from: suite, to: t_empty }
  - { from: t_empty, to: a_empty }
---
flowchart TD
    suite([persistence suite]) --> t_round[save then load] --> a_round([term/votedFor/log round-trip])
    suite --> t_resume[commit + persist + restore] --> a_resume([log intact, resumes])
    suite --> t_vote[grant vote + restore] --> a_vote([votedFor remembered])
    suite --> t_empty[load empty dir] --> a_empty([None, no error])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/raft.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Derive serde Serialize/Deserialize on RaftEntry; add a serializable PersistedState{term, voted_for, log}; add RaftNode::persisted() -> PersistedState and RaftNode::from_persisted(id, membership, state) so the node's durable state can be snapshotted and restored without changing the pure step-driven core."
  - path: projects/relay/src/raft_store.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "File-backed RaftStore: open(dir, node_id, FsyncPolicy), save(&PersistedState) writing term/votedFor + the log under data_dir and fsyncing per policy (hard state always durable), and load() -> Option<PersistedState> (None for an empty dir). No external dependency."
  - path: projects/relay/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare and re-export raft_store (RaftStore) and the new PersistedState type."
  - path: projects/relay/tests/raft_persistence.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Tests: PersistedState save/load round-trip; a sole-voter node persisted then restored via from_persisted keeps its committed log; votedFor is remembered across restore (no double-vote); loading an empty dir returns None."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] PersistedState{term,voted_for,log} via persisted()/from_persisted (pure core untouched); file-backed RaftStore.save fsyncs per FsyncPolicy and is called before the driver flushes the outbox => safety (no vote/ack before durable); load()->Option for empty dir. Sound.
- [unit-test] round-trip, sole-voter restore keeps committed log, votedFor remembered (no double-vote), empty-dir None.
- [changes] raft.rs serde+PersistedState, raft_store.rs, lib re-export, test. No external dep.
