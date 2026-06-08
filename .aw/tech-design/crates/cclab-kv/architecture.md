# cclab-kv Architecture

## Overview
<!-- type: overview lang: markdown -->

High-performance, multi-core key-value store with sharded in-memory storage, crash-safe WAL persistence, periodic snapshots, TCP server/client, and Redis-compatible data type operations.

## System Architecture
<!-- type: dependency lang: mermaid -->

```mermaid
classDiagram
    class engine {
        +KvEngine: sharded storage
        +Shard: single partition
        +Entry: value + metadata
        +EvictionPolicy: LRU/LFU/NoEviction
    }

    class types {
        +KvKey: validated key (max 256 chars)
        +KvValue: Int|Float|Decimal|String|Bytes|List|Map|Set|SortedSet|Null
    }

    class persistence {
        +PersistenceHandle: background thread
        +WalWriter / WalReader
        +SnapshotWriter / SnapshotLoader
        +RecoveryManager
        +format: WalOp, WalEntry, WalHeader, SnapshotHeader
    }

    class server {
        +KvServer: TCP listener
        +WaiterManager: blocking operations
        +protocol: command parsing
    }

    class client {
        +KvClient: TCP client
        +KvPool: connection pooling
        +protocol: command serialization
    }

    class metrics {
        +Metrics: counters + histograms
        +Counter: atomic u64
        +Histogram: bucket-based latency
    }

    class error {
        +KvError: operation errors
    }

    engine --> types : stores KvValue
    engine --> error : returns KvError
    persistence --> engine : snapshot/recovery
    persistence --> types : serializes KvValue
    server --> engine : handles commands
    server --> persistence : logs WAL ops
    server --> metrics : records operations
    client --> server : TCP protocol
```

## Data Flow
<!-- type: interaction lang: mermaid -->

```mermaid
sequenceDiagram
    participant C as Client
    participant S as KvServer
    participant E as KvEngine
    participant P as PersistenceHandle
    participant W as WalWriter (bg thread)
    participant SN as SnapshotWriter

    C->>S: SET key value [TTL]
    S->>E: set(key, value, ttl)
    E->>E: hash(key) mod num_shards -> shard_id
    E->>E: shard.set(key, entry)
    E-->>S: Ok(old_entry)
    S->>P: log_operation(WalOp::Set)
    P->>W: channel send (non-blocking)
    S-->>C: OK

    Note over W: Background thread
    W->>W: append entry to wal-current.log
    W->>W: batched fsync every 100ms
    W->>W: rotate when file > 1GB

    Note over SN: Periodic snapshot
    SN->>E: export_shard(0..N)
    SN->>SN: serialize + SHA256 checksum
    SN->>SN: atomic write (temp -> rename)
```

## Sharding Strategy
<!-- type: logic lang: mermaid -->

```mermaid
flowchart LR
    Key["KvKey"] --> Hash["FxHash (fast non-crypto)"]
    Hash --> Mod["hash & (num_shards - 1)"]
    Mod --> S0["Shard 0"]
    Mod --> S1["Shard 1"]
    Mod --> SN["Shard N-1"]
```

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| DEFAULT_NUM_SHARDS | 256 | Power of 2 for bitwise modulo |
| Shard lock | parking_lot::RwLock | Concurrent reads, exclusive writes |
| Hash function | std DefaultHasher (FxHash-like) | Fast, non-cryptographic |

## Entry Lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
stateDiagram-v2
    [*] --> Active: set() / setnx()
    Active --> Active: get() / incr() / hset() / cas()
    Active --> Expired: TTL elapsed
    Active --> [*]: delete()
    Expired --> [*]: cleanup_expired() / lazy on get()
    Active --> Active: expire(new_ttl)
    Active --> Active: persist() [remove TTL]
```

## Feature Summary
<!-- type: overview lang: markdown -->

| Feature | Description |
|---------|-------------|
| Sharded storage | 256 shards with RwLock for multi-core scalability |
| Value types | Int, Float, Decimal, String, Bytes, List, Map, Set, SortedSet, Null |
| TTL | Per-key expiration with lazy cleanup + periodic cleanup |
| CAS | Compare-and-swap with version tracking |
| Distributed locks | lock/unlock/extend_lock with owner ID |
| Eviction | AllKeysLru, VolatileLru, AllKeysLfu, NoEviction |
| WAL | Append-only with CRC32, batched fsync (100ms), rotation at 1GB |
| Snapshots | Periodic full state dump with SHA256, atomic writes |
| Recovery | Snapshot + WAL replay, skip corrupted entries |
| TCP server | Async TCP with custom protocol |
| Connection pool | Client-side pooling via KvPool |
| Metrics | Atomic counters + histograms for all operation types |

## Module Layout
<!-- type: overview lang: markdown -->

| Module | Files | Feature Gate |
|--------|-------|-------------|
| engine | engine.rs | default |
| types | types.rs | default |
| error | error.rs | default |
| metrics | metrics.rs | default |
| persistence | persistence/{mod, format, wal, snapshot, recovery, handle}.rs | default |
| server | server/{mod, server, protocol, waiter, main}.rs | default |
| client | client/{mod, client, pool, protocol}.rs | default |
