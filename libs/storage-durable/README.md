# storage-durable

## Brief

`storage-durable` defines shared durable local storage primitives for axiom
services: fsync policy, temp-file atomic replacement, CRC-framed append logs, and
sequence-named local snapshot stores. Services keep their own domain codecs and
state-machine semantics.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Shared Service Durability Contract | - | fsync policy, atomic writes, framed logs, and snapshot files |

### Shared Service Durability Contract

Services can compose one shared durable local storage layer instead of
reimplementing fsync, atomic rename, append-log frame parsing, torn-tail
recovery, or sequence-named snapshot retention locally.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `storage_durable`.
- Gate — behavior: `cargo test -p storage-durable` - durable file primitive
  coverage
- Gate: `cargo test -p storage-durable`
- Source: `libs/storage-durable/src/lib.rs`
- Evidence: `cargo test -p storage-durable`; libs/storage-durable/src/lib.rs
