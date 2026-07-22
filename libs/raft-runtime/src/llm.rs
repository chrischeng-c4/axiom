// SPEC-MANAGED: libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-llm-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! LLM topic provider for the shared raft-runtime contract.

/// Agent-facing topic describing raft-runtime topology and service boundaries.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "raft-runtime",
    summary: "Shared raft host topology, peer transport, snapshots, and read-your-write surface.",
    body: r#"# raft-runtime shared topic

## Ownership boundary
`raft-runtime` drives `raft-core` for a service-supplied `RaftStateMachine`. The
service owns command encoding, snapshot bytes, restore semantics, and API-level
routing. `raft-runtime` owns the tick/pump loop, h2c peer transport, peer router,
snapshot/compaction loop, local raft store, and read-your-write `propose`
surface. `FencedAssignment` supplies deterministic owner/epoch/expiry
transitions for service state machines that authorise external effects; it does
not execute effects or own application assignment keys.

## Auto-mode and topology env
Replica mode is automatic:

```text
REPLICAS_PER_SHARD > 1
```

Single-node mode is the default when the env is unset or `REPLICAS_PER_SHARD=1`.
Cluster mode reads the standard StatefulSet downward-API env:

```env
POD_NAME=<statefulset-name>-<ordinal>
SHARD_COUNT=<n>
REPLICAS_PER_SHARD=<n>
VOTER_COUNT=<n>
```

Pod ordinals map to topology slots with:

```text
shardIndex = ordinal % shardCount
replicaIndex = ordinal / shardCount
peerOrdinal = replicaIndex * shardCount + shardIndex
```

`ClusterTopology::from_env(prefix, headless_service, peer_port,
peers_override_env)` builds peer URLs from StatefulSet DNS and supports a
service-owned local override env such as `LUMEN_PEERS`.

## Reads, writes, and snapshots
`RaftHost::propose` returns after the command is applied locally, which gives
the caller a read-your-write boundary. Services that expose weaker reads should
make that explicit at their HTTP layer. Snapshot install and compaction are
hosted here; the snapshot payload remains service-owned.

## External-effect fencing
Effectful services store `FencedAssignment` beside their domain state. The
proposer resolves absolute times before encoding the command. Apply assigns an
owner and epoch; only after `propose` returns may an adapter execute the effect.
Expiry is another committed transition, and reassignment receives a higher
epoch so stale completions fail deterministically. Read-only services do not
need this layer.
"#,
};

/// Return the shared raft topic for CLI composition.
/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-llm-rs.md#source
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

#[cfg(test)]
mod tests {
    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "raft-runtime");
        assert!(topic.body.contains("RaftStateMachine"));
        assert!(topic.body.contains("REPLICAS_PER_SHARD > 1"));
    }
}
// CODEGEN-END
