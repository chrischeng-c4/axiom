# Cclab Util

## Brief

Cclab Util is the small shared Rust utility crate for formatting, collection
helpers, and bounded in-process caching.

It owns human-readable number/time/size formatting, deterministic slice helper
functions, and a dependency-light LRU cache with optional TTL.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Human Readable Formatting | - | number, ordinal, time, delta, and size formatting helpers |
| Iteration Helper Toolkit | - | chunking, windowing, dedupe, flattening, partitioning, and pairing helpers |
| LRU TTL Cache | - | pure Rust LRU cache with optional TTL and mutation helpers |

### Human Readable Formatting

Cclab Util provides human-readable formatting helpers for numbers, ordinals,
relative time, durations, and byte sizes so ecosystem crates can present
compact status and report text consistently.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_util::humanize`
- Gate — behavior: `cargo test -p cclab-util` - number, ordinal, time, delta,
  and size formatting behavior
- Gate: `cargo test -p cclab-util`
- Evidence: `cargo test -p cclab-util`

### Iteration Helper Toolkit

Cclab Util provides small deterministic slice and iterator helpers for common
collection transforms used across cclab crates.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_util::iter`
- Gate — behavior: `cargo test -p cclab-util` - chunked, windowed, first, one,
  unique, flatten, partition, pairwise, every_nth, and interleave behavior
- Gate: `cargo test -p cclab-util`
- Evidence: `cargo test -p cclab-util`

### LRU TTL Cache

Cclab Util provides a dependency-light LRU cache with optional TTL for
in-process runtime caches that need bounded size, update, lookup, eviction, and
expiry behavior.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_util::cache::LruCache`
- Gate — behavior: `cargo test -p cclab-util` - put/get, update, eviction,
  mutation, key listing, TTL expiry, and purge behavior
- Gate: `cargo test -p cclab-util`
- Evidence: `cargo test -p cclab-util`
