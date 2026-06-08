---
id: orbit-architecture
type: exploration
created_at: 2026-02-05T16:11:22.355382+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: orbit-architecture

## Scope
- #106: kqueue optimization (macOS/BSD native backend)
- #107: Memory allocator (slab/arena pattern)
- #110: Feature flags for modular builds

## Codebase Analysis

### Current Architecture

**Event Loop Backend (via Tokio)**
- Currently uses Tokio which internally handles epoll (Linux) and kqueue (macOS/BSD)
- Tokio already provides optimized backends, so #106 is about:
  - Exposing kqueue-specific tuning options
  - Optional direct kqueue access for advanced use cases
  - Benchmarking to validate Tokio's kqueue performance

**Memory Allocation**
- `TimerWheel` uses `BTreeMap<Instant, Vec<TimerEntry>>` - heap allocations per timer
- `WakerPool` uses `ArrayQueue<PythonWaker>` with capacity 256 - pre-allocated
- `HashedTimerWheel` uses `Vec<Mutex<Vec<TimerEntry>>>` - bucket-based
- Opportunity: Slab allocator for `TimerEntry` and `Handle` to reduce allocations

**Feature Flags (`Cargo.toml`)**
```toml
[features]
default = []
python = []  # Enable pyo3_bindings module
```
- Only `python` feature exists
- Need: `tls`, `unix-socket`, `dns`, `subprocess`, etc.

### Key Files

**1. Timer Wheel (`src/timer_wheel.rs`)**
- Line 72: `timers: Arc<Mutex<BTreeMap<Instant, Vec<TimerEntry>>>>`
- Candidate for slab allocation

**2. Handle (`src/handle.rs`)**
- Frequent allocation/deallocation
- Good candidate for arena pattern

**3. lib.rs**
- Lines 56-64: Conditional compilation already used for unix_socket
- Pattern to follow for feature flags

**4. Cargo.toml**
- Already has `[target.'cfg(unix)'.dependencies]` pattern
- Add similar for kqueue-specific deps

### Architecture Recommendations

**#106 kqueue Optimization**
- Tokio already uses kqueue on macOS/BSD
- Add optional `mio` kqueue tuning via feature flag
- Expose kqueue-specific settings (EVFILT flags, etc.)
- Create benchmark comparing with uvloop on macOS

**#107 Memory Allocator**
- Implement `Slab<T>` for fixed-size allocations
- Target structures:
  - `TimerEntry` (~48 bytes)
  - `Handle` (~16 bytes)
  - `ScheduledCallback` (~64 bytes)
- Pre-allocate pools based on expected load

**#110 Feature Flags**
```toml
[features]
default = ["python", "tls", "dns"]
python = []
tls = ["rustls"]  # or "native-tls"
dns = ["trust-dns-resolver"]
unix-socket = []
subprocess = []
kqueue-tuning = []  # macOS/BSD specific
slab-allocator = []
```

### Dependencies to Add

For kqueue:
- `mio` (optional, for direct access)

For slab allocator:
- `slab` crate OR custom implementation (user chose custom)

### Technical Considerations

1. **Backwards Compatibility**: Default features should match current behavior
2. **Compile Time**: More features = longer compile, use feature flags wisely
3. **Testing Matrix**: Need CI jobs for different feature combinations
4. **Documentation**: Document feature flag implications

## Spec Recommendations

| spec_id | spec_type | description |
|---------|-----------|-------------|
| feature-flags | utility | Modular feature flag system |
| slab-allocator | algorithm | Custom slab allocator for timer entries |
| kqueue-tuning | integration | macOS/BSD kqueue optimization |

## Impact Analysis

- Medium risk: Core allocation changes need careful testing
- Performance impact: Positive (reduced allocations, better locality)
- Breaking changes: None if defaults preserve current behavior

## Files to Modify
- `crates/cclab-orbit/Cargo.toml` - feature flags
- `crates/cclab-orbit/src/lib.rs` - conditional compilation
- `crates/cclab-orbit/src/timer_wheel.rs` - slab integration
- New: `crates/cclab-orbit/src/slab.rs` - custom allocator
