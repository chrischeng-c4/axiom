---
id: projects-lumen-src-wal_relay-rs
capability_refs:
  - id: "resilience"
    role: primary
    gap: "log-fan-out-rebuild-from-log"
    claim: "log-fan-out-rebuild-from-log"
    coverage: full
    rationale: "RelayWal is the optional relay-backed WAL implementation for lumen's log fan-out and rebuild-from-log capability."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/wal_relay.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/wal_relay.rs`, captured as a
per-file rust-source-unit for the relay-backed WAL backend.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `RelayWal` | projects/lumen/src/wal_relay.rs | struct | pub |
| `new` | projects/lumen/src/wal_relay.rs | function | pub |
| `publish` | projects/lumen/src/wal_relay.rs | function | pub |
| `subscribe` | projects/lumen/src/wal_relay.rs | function | pub |
| `latest_seq` | projects/lumen/src/wal_relay.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

```rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#rust-source-unit
// HANDWRITE-BEGIN gap="missing-generator:logic:54088576" tracker="pending-tracker" reason="RelayWal: a WalLog backed by relay's broadcast. publish POSTs to relay /v1/{subject}/publish (payload=json(WalRecord)); subscribe GETs /v1/{subject}/subscribe and decodes relay's length-prefixed CBOR LogEntry frames (relay::wire::decode_frames), mapping each to (seq+1, WalRecord). Plaintext h2c, no TLS."
pub struct RelayWal { /* see source */ }

impl RelayWal {
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    pub fn new(...) -> Result<Self> { /* see source */ }
}

#[async_trait]
impl WalLog for RelayWal {
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn publish(&self, record: WalRecord) -> Result<u64> { /* see source */ }

    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> { /* see source */ }

    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn latest_seq(&self) -> Result<u64> { /* see source */ }
}
// HANDWRITE-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: projects/lumen/src/wal_relay.rs
    action: claim
    section: rust-source-unit
    impl_mode: hand-written
    reason: "Feature-gated RelayWal implementation maps relay broadcast frames into lumen WAL records while generator primitives remain pending."
```
