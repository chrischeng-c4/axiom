//! Real `fileinput` for Mamba.
//!
//! fileinput is a stateful module: it iterates lines across a list of files
//! (or stdin), tracks per-file and cumulative line numbers, supports inplace
//! rewriting, custom open hooks, and binary/text modes. The previous native
//! stub returned empty lists / fixed sentinels from every helper and could
//! not provide a working `FileInput` class.
//!
//! Rather than special-casing the whole `FileInput` protocol in native Rust,
//! Mamba ships a pure-Python port (`py_src/fileinput.py`, adapted from
//! CPython 3.12) and lets Mamba's own compiler execute it. No native module
//! is registered here, so the vendored source is the only `fileinput` Mamba
//! sees: `import fileinput` resolves through `find_module()`'s search path.
//!
//! The source is embedded and materialized by the shared loader
//! (`vendor_lib.rs`, #867), which also serves `plistlib`, `mailbox`, and
//! `nturl2path`. This module used to run its own ad-hoc
//! `include_str!` + per-module temp-directory materialization; `register()`
//! is kept as a no-op call site (invoked from `stdlib::register_stdlib()`)
//! so the migration didn't need to touch that call list.

pub fn register() {
    // Intentionally empty: vendor_lib::register() (called earlier in
    // stdlib::register_stdlib()) already materializes py_src/fileinput.py
    // into the shared vendored search-path directory.
}
