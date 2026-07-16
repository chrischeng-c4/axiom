<!-- HANDWRITE-BEGIN gap="missing-generator:logic:fa386ab3" tracker="pending-tracker" reason="Tape leader-loss and durable replay survival contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-long-running-stability-resilience-survival-ec
summary: Tape Raft leader-loss and durable replay survival contract.
fill_sections: [e2e-test]
---

# EC: Stability Resilience Survival

Tape maps Lumen's survival category onto the replay journal's actual failure
boundary: Raft leader loss, snapshot catch-up, and durable committed-event
recovery.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-long-running-stability-raft-survival
    capability_id: long-running-stability
    claim_id: tape-raft-leader-loss-replay-survival
    contract_id: tape-raft-failover-and-snapshot-survival
    category: stability
    command: "cargo test -p tape --test raft_cluster --test raft_failover -- --test-threads=1"
    assertions:
      - "A three-node Tape group elects a leader, replicates ordered journal events, forwards follower writes, and continues after leader loss."
      - "A new Tape node catches up through snapshot installation without losing committed replay history."
      - "SIGKILL of the elected leader leaves surviving Tape nodes able to reelect with no committed-event loss."
      - "This contract proves replay survival; it does not claim search latency, packet-loss p99, or a completed live multi-shard soak."
```
<!-- HANDWRITE-END -->
