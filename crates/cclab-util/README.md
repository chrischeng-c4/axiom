# Cclab Util

## Brief

Cclab Util is the small shared Rust utility crate for formatting, collection
helpers, and bounded in-process caching.

It owns human-readable number/time/size formatting, deterministic slice helper
functions, and a dependency-light LRU cache with optional TTL.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Human Readable Formatting | - | implemented | passing | conformance | not_ready | number, ordinal, time, delta, and size formatting helpers |
| Iteration Helper Toolkit | - | implemented | passing | smoke | not_ready | chunking, windowing, dedupe, flattening, partitioning, and pairing helpers |
| LRU TTL Cache | - | implemented | passing | smoke | not_ready | pure Rust LRU cache with optional TTL and mutation helpers |

### Human Readable Formatting

ID: human-readable-formatting
Type: DeveloperTool
Surfaces: Rust API: `cclab_util::humanize`
EC Dimensions: behavior: `cargo test -p cclab-util` - number, ordinal, time, delta, and size formatting behavior
Root WI: -
Status: confirmed
Required Verification: smoke, conformance
Promise:
Cclab Util provides human-readable formatting helpers for numbers, ordinals, relative time, durations, and byte sizes so ecosystem crates can present compact status and report text consistently.
Gate Inventory: `cargo test -p cclab-util`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Humanize formatting contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-util` |

### Iteration Helper Toolkit

ID: iteration-helper-toolkit
Type: DeveloperTool
Surfaces: Rust API: `cclab_util::iter`
EC Dimensions: behavior: `cargo test -p cclab-util` - chunked, windowed, first, one, unique, flatten, partition, pairwise, every_nth, and interleave behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Util provides small deterministic slice and iterator helpers for common collection transforms used across cclab crates.
Gate Inventory: `cargo test -p cclab-util`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Iter helper behavior contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-util` |

### LRU TTL Cache

ID: lru-ttl-cache
Type: RuntimeTool
Surfaces: Rust API: `cclab_util::cache::LruCache`
EC Dimensions: behavior: `cargo test -p cclab-util` - put/get, update, eviction, mutation, key listing, TTL expiry, and purge behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Util provides a dependency-light LRU cache with optional TTL for in-process runtime caches that need bounded size, update, lookup, eviction, and expiry behavior.
Gate Inventory: `cargo test -p cclab-util`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| LRU and TTL cache behavior contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-util` |
