---
change: mamba-test-coverage
group: ffi-coverage
date: 2026-03-08
---

# Requirements

Achieve 100% line coverage for FFI subsystem (currently 74.4%, gap: 109 lines). Files: ffi/cbindgen.rs (0% — 29 lines), ffi/c_parser.rs (79.3%), ffi/stub_gen.rs (81%), ffi/safety.rs (83.3%), ffi/type_map.rs (88.3%). FFI is a safety-critical boundary — every code path must be tested.
