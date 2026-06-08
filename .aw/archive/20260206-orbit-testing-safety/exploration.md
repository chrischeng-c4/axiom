---
id: orbit-testing-safety
type: exploration
created_at: 2026-02-05T16:11:01.711071+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: orbit-testing-safety

## Scope
- #112: Fuzz testing with cargo-fuzz
- #113: Miri for undefined behavior detection

## Codebase Analysis

### Safety Status
The crate uses `#![forbid(unsafe_code)]` at the top of `lib.rs`, which means:
- No unsafe blocks anywhere in the crate
- Miri testing primarily validates dependencies (pyo3, tokio) rather than our code
- Fuzz testing remains valuable for finding logic bugs and edge cases

### Key Files for Fuzz Targets

**1. Timer Wheel (`src/timer_wheel.rs`)**
- `TimerWheel::register()` - timer registration
- `TimerWheel::cancel_timer()` - timer cancellation  
- `TimerWheel::process_expired()` - expiration handling
- Edge cases: past times, far future times, concurrent operations

**2. Hashed Timer Wheel (`src/timer_wheel_hashed.rs`)**
- Alternative timer implementation with O(1) operations
- Similar fuzz targets as basic timer wheel

**3. Waker (`src/waker.rs`)**
- `PythonWaker::wake()` - wake signaling
- `PythonWaker::reset()` - state reset for pooling
- `WakerPool::get()/put()` - pool operations
- Edge cases: double wake, reset after wake, concurrent access

**4. Handle (`src/handle.rs`)**
- `Handle::cancel()` - cancellation flag
- `TimerHandle` operations
- Edge cases: cancel-before-fire, cancel-after-fire

**5. Completion Tracking (`src/completion.rs`)**
- `WaitSet` operations
- `CompletionNotifier` signaling

### Architecture Considerations

**Fuzz Testing Setup**
```
crates/cclab-orbit/
├── fuzz/
│   ├── Cargo.toml
│   └── fuzz_targets/
│       ├── fuzz_timer_wheel.rs
│       ├── fuzz_waker.rs
│       └── fuzz_handle.rs
```

**Miri Integration**
- Add `cargo miri test` to CI
- Miri primarily useful for validating atomic ordering
- Current code uses AtomicBool with proper Ordering (Release/Acquire)

### Dependencies
- `cargo-fuzz` for fuzz testing
- Miri is built into rustup (nightly)

### Technical Considerations

1. **Python/PyO3 Isolation**: Fuzz targets need to avoid PyO3 code since it requires Python runtime. Use `#[cfg(not(fuzzing))]` or separate pure-Rust test modules.

2. **Tokio Runtime**: Fuzz targets may need mock/minimal runtime or use `tokio-test`.

3. **Arbitrary Input**: Use `arbitrary` crate for structured input generation.

## Spec Recommendations

| spec_id | spec_type | description |
|---------|-----------|-------------|
| fuzz-targets | utility | Fuzz testing infrastructure and targets |
| miri-ci | utility | Miri integration in CI pipeline |

## Impact Analysis

- Low risk: Testing-only changes, no production code modifications
- CI time increase: Fuzz testing can be run separately, Miri adds ~2-3x test time
- Coverage improvement: Better edge case coverage

## Files to Create
- `crates/cclab-orbit/fuzz/Cargo.toml`
- `crates/cclab-orbit/fuzz/fuzz_targets/*.rs`
- `.github/workflows/` updates for Miri
