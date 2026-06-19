// HANDWRITE-BEGIN gap="missing-generator:unit-test:b72bd304" tracker="pending-tracker" reason="Deterministic in-process simulation: a message bus pumps node outboxes to handlers. Tests leader election, replicate+commit ordering, kill-leader -> re-elect with no committed loss, learner replicates/applies but never votes nor counts toward majority, stale higher-term step-down, and a relay-integration scenario (command=publish, apply=relay engine) that converges across a leader failover."
// TODO: hand-write content for `projects/relay/tests/raft_core.rs`.
// HANDWRITE-END
