<!-- HANDWRITE-BEGIN gap="missing-generator:logic:09f0e874" tracker="pending-tracker" reason="Tape restart/recovery and admission stability contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-long-running-stability-replay-resilience-ec
summary: Tape restart, recovery, and admission stability contract for the replay journal.
fill_sections: [e2e-test]
---

# EC: Long-Running Replay Stability

This is the Tape adaptation of Lumen's long-running resilience category. It
does not claim Lumen's search p99, packet-loss, FD-leak, or live-soak evidence;
those need a Tape-specific workload and remain explicit follow-ups.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-long-running-stability-replay-restart
    capability_id: long-running-stability
    claim_id: tape-append-checkpoint-restart-recovery
    contract_id: tape-replay-stability-restart-recovery
    category: stability
    command: "cargo test -p tape --test long_running_stability --test raft_persistence -- --nocapture"
    assertions:
      - "Repeated Raft restart preserves committed append history and consumer checkpoint progress."
      - "A restarted node recovers its applied floor and accepts new proposals without replaying an already persisted checkpoint."
  - id: tape-long-running-stability-admission
    capability_id: long-running-stability
    claim_id: tape-write-admission-boundary
    contract_id: tape-replay-stability-admission
    category: stability
    command: "cargo test -p tape --test service_admission -- --nocapture"
    assertions:
      - "Append requests are classified as write admission while the default router remains explicitly unbounded until a configured shared admission policy is supplied."
      - "No live soak, retention, compaction, packet-loss, or resource-leak threshold is claimed by this baseline contract."
```
<!-- HANDWRITE-END -->
