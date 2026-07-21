---
id: '2221'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-primary-replicas-raft-backed-task-scheduler-verification
entry: elect_primary
nodes:
  elect_primary: { kind: start, label: "start a three-voter Defer cluster over shared raft runtime and peer transport" }
  commit_task: { kind: process, label: "configure the queue and create a committed scheduled task through the elected primary" }
  follower_lease: { kind: process, label: "lease due work through a follower so the committed attempt records that follower as executor with a fence epoch" }
  first_fence: { kind: decision, label: "does a non-owner ack fail while the committed owner ack succeeds and every replica converges to Succeeded?" }
  lose_primary: { kind: process, label: "lease a second task, lose the active leader with a live lease, and elect a surviving quorum leader" }
  recover_lease: { kind: process, label: "reject the stale ack, reclaim the expired lease, reassign with a higher epoch and new attempt id, and commit the terminal outcome" }
  restart_replica: { kind: process, label: "restart the first failed primary from the same durable directory and wait for committed task-state catch-up before another leader loss" }
  snapshot_recovery: { kind: process, label: "drive retry to DeadLettered and verify every replica reports identical post-restart queue snapshot counts for task, terminal, scheduled, and in-flight state" }
  second_failover: { kind: process, label: "remove the current leader after catch-up and complete another create lease ack lifecycle through the remaining quorum" }
  peer_mtls: { kind: process, label: "replicate scheduler state with trusted peer mTLS material, then present attacker-CA client and server identities" }
  peer_ok: { kind: decision, label: "do trusted peers converge while attacker identities are rejected before the Raft router handles a request?" }
  fail: { kind: terminal, label: "primary-replicas claim fails closed" }
  verified: { kind: terminal, label: "raft-backed task scheduler claim is externally verified" }
edges:
  - { from: elect_primary, to: commit_task }
  - { from: commit_task, to: follower_lease }
  - { from: follower_lease, to: first_fence }
  - { from: first_fence, to: lose_primary, label: "yes" }
  - { from: first_fence, to: fail, label: "no" }
  - { from: lose_primary, to: recover_lease }
  - { from: recover_lease, to: restart_replica }
  - { from: restart_replica, to: snapshot_recovery }
  - { from: snapshot_recovery, to: second_failover }
  - { from: second_failover, to: peer_mtls }
  - { from: peer_mtls, to: peer_ok }
  - { from: peer_ok, to: verified, label: "yes" }
  - { from: peer_ok, to: fail, label: "no" }
---
flowchart TD
    elect_primary([start three-voter Defer cluster]) --> commit_task[commit queue config and first scheduled task]
    commit_task --> follower_lease[follower forwards lease with committed executor fence]
    follower_lease --> first_fence{stale ack rejected and owner ack converges?}
    first_fence -->|yes| lose_primary[lose leader with live lease]
    first_fence -->|no| fail([fail closed])
    lose_primary --> recover_lease[expire reclaim and reassign with higher epoch]
    recover_lease --> restart_replica[restart failed primary from same durable directory]
    restart_replica --> snapshot_recovery[verify dead-letter snapshot counts converge]
    snapshot_recovery --> second_failover[lose a second leader after catch-up]
    second_failover --> peer_mtls[test trusted and attacker-CA peer identities]
    peer_mtls --> peer_ok{replication succeeds only for trusted peers?}
    peer_ok -->|yes| verified([primary-replicas claim externally verified])
    peer_ok -->|no| fail
```
