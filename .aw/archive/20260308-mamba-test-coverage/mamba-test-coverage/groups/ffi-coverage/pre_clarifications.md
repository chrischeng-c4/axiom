---
change: mamba-test-coverage
group: ffi-coverage
date: 2026-03-08
status: answered
---

# Pre-Clarifications

### Q1: cbindgen-testability
- **Answer**: ffi/cbindgen.rs (74 lines) is actively used code. It shells out to the `cbindgen` CLI tool and orchestrates the full FFI pipeline (cbindgen → parse → map types → generate stubs). Can be tested: 1) `crate_name_from_dir` is pure logic, easy to unit test. 2) `run_cbindgen` can be tested with a temp crate dir — if cbindgen is not installed, test should gracefully handle the error path. 3) `generate_ffi_bindings` is an integration test covering the full pipeline.

### Q2: ffi-safety-edge-cases
- **Answer**: Test the guard logic with controlled inputs — verify that safety checks return correct results for valid/invalid inputs. No need for actual unsafe memory scenarios (double-free etc.) since those would be UB. Focus on: bounds checking logic, null pointer guard returns, type validation in marshalling, and error paths when invalid types are passed.

