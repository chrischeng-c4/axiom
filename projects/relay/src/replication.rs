// HANDWRITE-BEGIN gap="missing-generator:logic:867abe72" tracker="pending-tracker" reason="spawn_follower(local, leader_url, subjects) -> FollowerHandle: one tokio task per subject that tails the leader's subscribe stream over h2c, decodes length-prefixed CBOR LogEntry frames, and re-applies each via local.publish (idempotent on message_id); reconnects with backoff on stream end/error. FollowerHandle aborts the tasks on stop/drop."
// TODO: hand-write content for `projects/relay/src/replication.rs`.
// HANDWRITE-END
