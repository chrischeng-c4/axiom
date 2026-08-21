//! Real `plistlib` for Mamba.
//!
//! plistlib is large, stateful, and full of rich-dunder value types (the `UID`
//! class, the `InvalidFileException` hierarchy, the `PlistFormat` enum-likes)
//! plus two complete serialization formats (Apple XML and binary `bplist00`).
//! Re-implementing all of that as native Rust dispatchers would mean
//! special-casing every dunder in `class.rs`. Instead Mamba ships a
//! pure-Python port (`py_src/plistlib.py`) and lets Mamba's own compiler
//! execute it.
//!
//! The old long_tail stub (which returned empty strings/dicts from every
//! dump/load call) is removed in favour of this module.
//!
//! The source is embedded and materialized by the shared loader
//! (`vendor_lib.rs`, #867), which also serves `fileinput`, `mailbox`, and
//! `nturl2path`. This module used to run its own ad-hoc `include_str!` +
//! per-module temp-directory materialization; `register()` is kept as a
//! no-op call site (invoked from `stdlib::register_stdlib()`) so the
//! migration didn't need to touch that call list.

pub fn register() {
    // Intentionally empty: vendor_lib::register() (called earlier in
    // stdlib::register_stdlib()) already materializes py_src/plistlib.py
    // into the shared vendored search-path directory.
}
