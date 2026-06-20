---
id: relay-raft-core
summary: Self-contained single-shard Raft consensus core (no external dep). Step-driven RaftNode (tick + message handlers, deterministic) doing leader election, log replication with log-matching + truncate-conflicting-suffix, commit-by-majority over voters only, and apply of committed command entries. Auto voter/learner membership from N and ordinal — voters = largest-odd<=N, the trailing even node is a non-voting learner. RSM model — committed entries are surfaced for the relay layer to apply to its engine, while relay's append-only durable log is untouched. RaftTransport trait + in-process impl for deterministic tests.
capability_refs:
  - id: ha-replication
    role: primary
    gap: in-process-raft-convergence
    claim: in-process-raft-convergence
    coverage: full
    rationale: "Defines the deterministic in-process raft state machine, election, replication, commit ordering, failover, and relay-engine convergence contract."
fill_sections: [logic, unit-test, changes]
---

# relay single-shard Raft consensus core (self-contained, RSM, auto voter/learner)

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-raft-core-flow
entry: tick
nodes:
  tick:
    kind: start
    label: "RaftNode driven by tick() + handle(from, msg); deterministic, no hidden timers"
  role:
    kind: decision
    label: "role?"
  elect:
    kind: process
    label: "Follower/Candidate voter: election_elapsed >= timeout -> term+1, vote self, RequestVote to voter peers; majority of VOTERS grants -> Leader (learner never starts an election)"
  lead:
    kind: process
    label: "Leader: on propose append RaftEntry{term,index,command}; replicate via AppendEntries(prev_log_index/term, entries, leader_commit) to all peers (voters + learners)"
  match:
    kind: decision
    label: "follower AppendEntries: prevLogIndex/Term match?"
  reject:
    kind: process
    label: "mismatch -> reply false; leader backs off next_index and retries"
  accept:
    kind: process
    label: "match -> truncate any conflicting suffix, append entries, commit_index = min(leader_commit, last)"
  commit:
    kind: process
    label: "leader advances commit_index to largest N with majority of VOTERS' match_index >= N and log[N].term == current_term (learners never counted)"
  apply:
    kind: terminal
    label: "committed entries (last_applied+1..=commit_index) surfaced via take_committed() -> relay layer applies each command to its engine; all nodes converge"
  stepdown:
    kind: process
    label: "any message with higher term -> become follower of that term (stale leader steps down)"
edges:
  - { from: tick, to: role }
  - { from: role, to: elect, label: "follower/candidate" }
  - { from: role, to: lead, label: "leader" }
  - { from: elect, to: lead, label: "won majority" }
  - { from: lead, to: match, label: "AppendEntries" }
  - { from: match, to: reject, label: "no" }
  - { from: match, to: accept, label: "yes" }
  - { from: accept, to: commit }
  - { from: commit, to: apply }
  - { from: reject, to: lead, label: "retry lower next_index" }
  - { from: role, to: stepdown, label: "higher term seen" }
  - { from: stepdown, to: apply }
---
flowchart TD
    tick([tick + handle msg]) --> role{role?}
    role -->|follower/candidate voter| elect[election timeout -> RequestVote; majority of voters -> Leader]
    role -->|leader| lead[propose append + AppendEntries to peers]
    elect -->|won| lead
    lead --> match{prevLog match?}
    match -->|no| reject[reply false; leader backs off next_index]
    match -->|yes| accept[truncate conflict, append, commit=min leaderCommit,last]
    accept --> commit[leader commit when majority of voters match + current-term entry]
    commit --> apply([take_committed -> relay applies; converge])
    reject --> lead
    role -->|higher term| stepdown[step down to follower]
    stepdown --> apply
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-raft-core-test-plan
entry: suite
nodes:
  suite: { kind: start, label: "in-process deterministic Raft simulation (a message bus pumps outboxes to handlers)" }
  t_elect: { kind: process, label: "3 voters; tick until a leader emerges" }
  a_elect: { kind: terminal, label: "assert exactly one Leader, two Followers, same term" }
  t_repl: { kind: process, label: "leader proposes commands; pump" }
  a_repl: { kind: terminal, label: "assert all nodes commit them in order and take_committed yields the same sequence" }
  t_kill: { kind: process, label: "commit entries, then drop the leader from the bus; tick survivors" }
  a_kill: { kind: terminal, label: "assert a new leader is elected and every committed entry still present (no committed loss); new entries commit again" }
  t_learner: { kind: process, label: "N=4 -> voters {0,1,2}, learner {3}; elect + propose" }
  a_learner: { kind: terminal, label: "assert learner replicates + applies committed entries but never becomes candidate nor counts toward majority (majority stays 2 of 3)" }
  t_stale: { kind: process, label: "deliver a stale higher-term message to a leader" }
  a_stale: { kind: terminal, label: "assert it steps down to follower of the higher term" }
  t_relay: { kind: process, label: "wire command=publish, apply=relay engine; publish via leader, pump, kill leader, publish more" }
  a_relay: { kind: terminal, label: "assert every surviving node's relay engine holds the same committed messages" }
edges:
  - { from: suite, to: t_elect }
  - { from: t_elect, to: a_elect }
  - { from: suite, to: t_repl }
  - { from: t_repl, to: a_repl }
  - { from: suite, to: t_kill }
  - { from: t_kill, to: a_kill }
  - { from: suite, to: t_learner }
  - { from: t_learner, to: a_learner }
  - { from: suite, to: t_stale }
  - { from: t_stale, to: a_stale }
  - { from: suite, to: t_relay }
  - { from: t_relay, to: a_relay }
---
flowchart TD
    suite([deterministic sim bus]) --> t_elect[3 voters tick] --> a_elect([one leader, same term])
    suite --> t_repl[propose] --> a_repl([all commit in order])
    suite --> t_kill[drop leader] --> a_kill([re-elect, no committed loss])
    suite --> t_learner[N=4] --> a_learner([learner applies, never votes/counts])
    suite --> t_stale[higher-term msg] --> a_stale([leader steps down])
    suite --> t_relay[command=publish] --> a_relay([engines converge across failover])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/src/raft.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: "Self-contained step-driven Raft: RaftNode (roles, current_term, voted_for, in-memory log of RaftEntry{term,index,command}, commit_index/last_applied, per-peer next/match_index), tick() + handle(from, RaftMsg) doing election/replication/commit/step-down, propose(command), take_outgoing(), take_committed(); auto_membership(n) -> voters=largest-odd<=N + trailing learner; majority counts voters only; learners replicate+apply but never vote/start elections. RaftTransport trait + an in-process bus impl for tests. No external dependency."
  - path: projects/relay/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Declare and re-export the raft module (RaftNode, RaftEntry, RaftMsg, Membership, auto_membership, RaftTransport)."
  - path: projects/relay/tests/raft_core.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Deterministic in-process simulation: a message bus pumps node outboxes to handlers. Tests leader election, replicate+commit ordering, kill-leader -> re-elect with no committed loss, learner replicates/applies but never votes nor counts toward majority, stale higher-term step-down, and a relay-integration scenario (command=publish, apply=relay engine) that converges across a leader failover."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Step-driven RaftNode (tick + handle): election (term bump, self-vote, RequestVote, majority of VOTERS), replication (AppendEntries with prevLog matching + truncate conflicting suffix), commit-by-majority restricted to current-term entries over voters only, step-down on higher term, take_committed apply. auto_membership(N) = largest-odd voters + trailing learner; learners replicate/apply but never vote/start elections/count. RaftTransport trait + in-process bus. RSM model leaves relay's append-only log untouched. Self-contained, sound.
- [unit-test] Deterministic sim: elect, replicate+commit order, kill-leader no-committed-loss + re-elect, learner non-voting, stale step-down, relay-integration failover convergence.
- [changes] raft.rs + lib re-export + raft_core.rs. No external dep.
