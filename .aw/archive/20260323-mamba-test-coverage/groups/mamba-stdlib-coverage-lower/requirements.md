---
change: mamba-test-coverage
group: mamba-stdlib-coverage-lower
date: 2026-03-23
---

# Requirements

Add inline `#[cfg(test)]` test modules to the 10 lowest-coverage stdlib files in `crates/mamba/src/runtime/stdlib/`, raising each from near-zero to 100% line and branch coverage as measured by `cargo-llvm-cov`.

Target files (ordered by current coverage):
| File | Coverage |
|------|----------|
| queue_mod.rs | 4% |
| statistics_mod.rs | 5% |
| shlex_mod.rs | 7% |
| calendar_mod.rs | 8% |
| locale_mod.rs | 10% |
| lzma_mod.rs | 11% |
| zlib_mod.rs | 11% |
| secrets_mod.rs | 12% |
| bisect_mod.rs | 14% |
| abc_mod.rs | 14% |

For each file the change-spec must:
1. Enumerate every public function and impl block in the source module.
2. Map each function/method to its branch paths (if/else, match arms, early returns, error paths).
3. Specify exact test cases — inputs, expected outputs, and which branch each test exercises.
4. Target 100% line + 100% branch coverage (as reported by `cargo llvm-cov --branch`).

Constraints:
- Tests must be self-contained (no network, no filesystem writes outside temp dirs).
- lzma_mod and zlib_mod tests must use fixed in-memory byte arrays, not external files.
- queue_mod concurrency tests must use `std::thread` or `tokio::task` matching the existing runtime pattern in the crate.
- All 10 files are in the same crate (`crate:mamba`); changes are atomic and ship in one PR.
