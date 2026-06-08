---
id: orbit-p2-enhancements
type: exploration
created_at: 2026-02-05T13:26:43.590377+00:00
needs_clarification: false
---

# Codebase Exploration

# Orbit P2 Enhancements Exploration

## Codebase Analysis

### Orbit Crate Structure (`crates/cclab-orbit/`)
```
src/
├── lib.rs              # Crate entry
├── loop_impl.rs        # PyLoop implementation (asyncio event loop)
├── debug.rs            # Debug mode implementation ✓
├── executor.rs         # Task executor
├── task.rs             # Task management
├── waker.rs            # Python waker bridge
├── timer_wheel.rs      # Timer wheel implementation
├── handle.rs           # Handle types
├── future.rs           # Future bridge
├── network.rs          # TCP/UDP networking
├── dns.rs              # DNS resolution
├── tls.rs              # TLS support
├── signal.rs           # Signal handling
├── subprocess.rs       # Subprocess support
└── pyo3_bindings/      # Python bindings
    └── mod.rs
```

### Existing Implementation Status

| Issue | Component | Current State |
|-------|-----------|---------------|
| #69 Integration Tests | `tests/` | ❌ Does not exist |
| #70 Benchmarks | `benches/` | ❌ Does not exist |
| #71 Stress Tests | `tests/` | ❌ Does not exist |
| #72 Bridge Docs | `knowledge/orbit/bridge-internals.md` | ✅ Exists (basic) |
| #73 Tuning Guide | `knowledge/orbit/performance-tuning.md` | ✅ Exists (basic) |
| #74 Debug Mode | `src/debug.rs` | ⚠️ Partial |

### Debug Mode Analysis (#74)

**Rust Core** (`debug.rs`):
- `DebugConfig`: Configuration struct ✅
- `DebugMonitor`: Statistics tracking ✅
- `SlowCallback`: Slow callback records ✅
- `LoopStatistics`: Loop stats ✅
- Unit tests included ✅

**Python API** (`loop_impl.rs`):
- `set_debug(enabled: bool)` ✅ Exposed
- `get_debug() -> bool` ✅ Exposed
- `get_debug_stats()` ❌ NOT exposed
- `DebugMonitor` not integrated with `PyLoop`

**Gap**: The `DebugMonitor` class exists but is not connected to `PyLoop`. Need to:
1. Add `SharedDebugMonitor` field to `PyLoop`
2. Expose `get_debug_stats()` to Python
3. Integrate callback timing with debug monitor

### Testing Infrastructure

**cclab-probe** provides:
- `benchmark.rs` - Benchmark framework
- `performance/` - Performance testing
- `runner.rs` - Test runner
- PyO3 bindings for Python test integration

Can be used as foundation for orbit tests.

## Impact Analysis

### Files to Create
1. `crates/cclab-orbit/tests/integration_tests.rs` - Integration tests
2. `crates/cclab-orbit/benches/benchmarks.rs` - Criterion benchmarks  
3. `crates/cclab-orbit/tests/stress_tests.rs` - Stress tests

### Files to Modify
1. `src/loop_impl.rs` - Add DebugMonitor integration
2. `src/debug.rs` - Add Python-friendly stats output
3. `Cargo.toml` - Add dev-dependencies (criterion, tokio-test)

### Documentation to Enhance
1. `knowledge/orbit/bridge-internals.md` - Add waker details, error handling
2. `knowledge/orbit/performance-tuning.md` - Add benchmark results

## Spec Recommendations

| Spec ID | Type | Description | Depends On |
|---------|------|-------------|------------|
| `debug-api` | utility | Debug mode Python API enhancement | - |
| `integration-tests` | utility | Integration test suite | debug-api |
| `benchmarks` | utility | Performance benchmark suite | - |
| `stress-tests` | utility | High-concurrency stress tests | integration-tests |
| `bridge-docs` | utility | Bridge documentation enhancement | debug-api |
| `tuning-guide` | utility | Tuning guide enhancement | benchmarks |

## Risk Assessment

1. **Low Risk**: Documentation enhancements - straightforward additions
2. **Low Risk**: Benchmarks - isolated, no production impact
3. **Medium Risk**: Debug API - modifying PyLoop struct, need careful testing
4. **Medium Risk**: Stress tests - may expose hidden race conditions

## Technical Considerations

1. **Test Framework**: Use Rust integration tests + cclab-probe for Python
2. **Benchmark Tool**: Use Criterion for Rust, pyperf for Python comparison
3. **Comparison Targets**: uvloop, asyncio stdlib
4. **CI Integration**: Tests must complete within reasonable time (&lt;5 min)
