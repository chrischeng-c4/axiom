---
change: mamba-native-stdlib
group: mamba-native-stdlib-rewrite
date: 2026-03-07
---

# Requirements

1. Dependency Integration: Update crates/mamba/Cargo.toml to include robust Rust standard library equivalents (e.g., serde_json, regex, chrono, rusqlite, base64, sha2, rand, etc.).
2. Implementation Rewrite: Replace the manual, stubbed, or minimal implementations in crates/mamba/src/runtime/stdlib/*.rs with production-grade Rust code utilizing these crates.
    - json_mod.rs: Replace manual parser with serde_json.
    - re_mod.rs: Replace literal search with regex crate.
    - datetime_mod.rs: Replace manual date math with chrono.
    - sqlite3_mod.rs: Replace in-memory HashMap stub with rusqlite.
3. Builtins Completion: Fill in or improve stubs in crates/mamba/src/runtime/builtins.rs (e.g., mb_eval, mb_exec, mb_globals).
4. Registration Refinement: Refactor symbols.rs and the module registration process to be more automated and less error-prone (e.g., using a registry macro).
5. Error Handling: Ensure Rust-level errors (e.g., IoError, SerdeError) are properly mapped to Mamba exceptions (MbException).
