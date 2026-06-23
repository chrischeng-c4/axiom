//! `loom controller` — the scheduler + sharded, strongly-consistent DAG state.
//!
//! Picks the next ready node, assembles input refs from keep, publishes the
//! task to relay, observes acks, advances DAG state, and counts fan-in
//! barriers. Also serves the thin client control API (`POST /runs`,
//! `GET /runs/{id}`, `GET /runs/{id}/result-ref`). Implemented in P2 (#106);
//! state durability per #110 / #123.

/// Entry point for `loom controller`.
pub fn run() -> anyhow::Result<()> {
    anyhow::bail!("loom controller: not yet implemented (epic #106, P2 scheduler loop)")
}
