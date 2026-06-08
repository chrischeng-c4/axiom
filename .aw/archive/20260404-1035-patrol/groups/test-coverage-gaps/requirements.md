---
change: 1035-patrol
group: test-coverage-gaps
date: 2026-04-04
---

# Requirements

Add tests to increase per-module line coverage for the 29 files currently below 50% coverage. Focus on the lowest-coverage files first (queue_mod 4%, statistics_mod 5%, shlex_mod 7%, etc.). Target: bring each file to at least 50% line coverage in this iteration. Add inline #[cfg(test)] module tests within each file. Priority on core modules (ffi/c_types 0%, driver/mod 33%, codegen/cranelift/mod 45%) over stdlib stubs.
