# Cclab Kv

## Brief

Cclab Kv is the Rust key-value storage engine and TCP service for cclab
runtime components.

It owns the sharded in-process KV engine, typed value model, Redis-like
operations, async TCP client/pool, `kv-server` binary, binary wire protocol,
and WAL/snapshot persistence path. The README capability map separates the
library engine, service/client protocol, and persistence/recovery contracts so
agents can route implementation and verification work to the right surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Sharded Engine And Data Types | - | implemented | passing | conformance | not_ready | sharded typed KV operations with TTL, CAS, locks, batches, collections, and eviction |
| TCP Server Client Protocol | - | implemented | passing | conformance | not_ready | `kv-server`, async client/pool, binary protocol, namespaces, and remote operations |
| Persistence Snapshot And Recovery | - | implemented | passing | conformance | not_ready | WAL, snapshot, background persistence, crash recovery, and server persistence flags |

### Sharded Engine And Data Types

ID: sharded-engine-and-data-types
Type: RuntimeTool
Surfaces:
- Rust API: `cclab_kv::{KvEngine, KvKey, KvValue, EvictionPolicy}` - in-process sharded KV engine surface.
EC Dimensions:
- behavior: `cargo test -p cclab-kv` - set/get/delete/exists, TTL, CAS, counters, locks, hash/list/batch operations, sharding, eviction, and value encoding.
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Kv provides an in-process sharded key-value engine with validated keys, typed values, TTL management, atomic counters, CAS, distributed lock primitives, batch operations, Redis-style hash/list APIs, and eviction policies.
Gate Inventory:
- `cargo test -p cclab-kv`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Sharded engine behavior contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-kv` |

### TCP Server Client Protocol

ID: tcp-server-client-protocol
Type: Service
Surfaces:
- CLI: `kv-server` - starts the TCP KV service.
- Rust API: `cclab_kv::{KvServer, KvClient, KvPool, PoolConfig}` - async service, client, and pool surface.
EC Dimensions:
- behavior: `cargo test -p cclab-kv` - protocol encoding, client operations, pool behavior, waiter behavior, and namespace handling.
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Kv exposes a TCP service and async Rust client surface for remote key-value operations, client-side namespaces, connection pooling, binary protocol encoding, and blocking list waiters.
Gate Inventory:
- `cargo test -p cclab-kv`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| TCP service and client protocol contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-kv` |

### Persistence Snapshot And Recovery

ID: persistence-snapshot-and-recovery
Type: RuntimeTool
Surfaces:
- Rust API: `cclab_kv::persistence::{PersistenceConfig, PersistenceHandle, recovery::RecoveryManager}` - persistence configuration, handle, and recovery API.
- CLI: `kv-server --data-dir --disable-persistence --fsync-interval-ms --snapshot-interval-secs --snapshot-ops-threshold` - server persistence configuration.
EC Dimensions:
- behavior: `cargo test -p cclab-kv` - WAL format, checksums, snapshot files, background persistence, recovery cycle, batch recovery, lock recovery, and recovery performance smoke.
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Kv persists engine mutations through background WAL and snapshot management, recovers state by loading snapshots and replaying WAL deltas, and exposes server flags for persistence mode, data directory, fsync cadence, and snapshot thresholds.
Gate Inventory:
- `cargo test -p cclab-kv`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Persistence and recovery contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-kv` |
