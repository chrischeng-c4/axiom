//! Shared vendored CPython 3.12 `Lib/` subtree loader (#867).
//!
//! Some stdlib modules are large, stateful, and full of rich-dunder value
//! types that would mean special-casing every dunder in a native Rust
//! dispatcher (see `plistlib_mod`/`fileinput_mod`'s original doc comments).
//! For those, Mamba ships a pure-Python port adapted from real CPython 3.12
//! source and lets Mamba's own compiler execute it, instead of hand-rolling
//! a native shell.
//!
//! Previously each such module carried its own ad-hoc `include_str!` +
//! per-module temp-directory materialization (`fileinput_mod`, `plistlib_mod`,
//! and `long_tail_mod::register_mailbox`), each writing to its own
//! content-hashed directory and each calling `mb_insert_search_path`
//! separately. This module is the single shared mechanism those three were
//! migrated to: every vendored source file is embedded at compile time,
//! materialized ONCE (together) to one shared temp directory at startup, and
//! that one directory is inserted into the import search path so
//! `find_module()` (see `module.rs`) resolves `import X` to the vendored
//! source.
//!
//! ## Precedence
//!
//! `register()` is called once from `stdlib::register_stdlib()`, which runs
//! before `mb_init_search_paths()` reads `PYTHONPATH` (see `main.rs` /
//! `driver/mod.rs`). Combined with `find_module`'s search order, the net
//! precedence for `import X` is:
//!
//! 1. **native** — a module already pre-registered in `MODULES` (e.g. via
//!    `mb_module_register`) is returned straight from `mb_import`'s cache
//!    check; `find_module` (and therefore this tree) is never consulted.
//! 2. **script-dir** — the directory of the currently executing script
//!    (`SCRIPT_DIR`), checked first inside `find_module`.
//! 3. **vendored** — this tree, inserted at `SEARCH_PATHS[0]` (ahead of the
//!    default `"."` entry and anything `mb_init_search_paths` adds later).
//! 4. **user** — the default `"."` entry and any `PYTHONPATH` additions.
//!
//! Adding a module: embed its source below and add a `(name, SRC)` entry to
//! `VENDORED_MODULES`. No other call site changes are needed.

use std::io::Write;
use std::path::PathBuf;

/// `(module_name, embedded source)`. `module_name` becomes `<name>.py` on disk.
const VENDORED_MODULES: &[(&str, &str)] = &[
    ("fileinput", include_str!("py_src/fileinput.py")),
    ("plistlib", include_str!("py_src/plistlib.py")),
    ("mailbox", include_str!("py_src/mailbox.py")),
    // Proves the loader resolves a module Mamba has NO native shell for at
    // all (as opposed to fileinput/plistlib/mailbox, which replaced a native
    // stub) — see #867 AC2.
    ("nturl2path", include_str!("py_src/nturl2path.py")),
];

/// Materialize the vendored tree (once) and add it to the import search path.
///
/// Deliberately does NOT register a native stub for any of these names: a
/// registered module is pre-seeded into `MODULES` and would win the import
/// cache before `find_module()` is ever consulted, permanently shadowing the
/// real source. With no stub, `import fileinput` (etc.) falls through to the
/// search path and loads the vendored `.py` file. A user-supplied module of
/// the same name in the running script's directory still wins regardless of
/// registration order: `find_module()` consults `SCRIPT_DIR` before
/// `SEARCH_PATHS`.
pub fn register() {
    if let Some(dir) = materialize_vendor_tree() {
        super::super::module::mb_insert_search_path(0, &dir.display().to_string());
    }
}

fn materialize_vendor_tree() -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    // Content-address the directory by hashing every embedded source
    // together, so the tree is (re)written at most once per build and
    // concurrent mamba processes safely share it — the same property the
    // three original per-module materializers each provided individually.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (name, src) in VENDORED_MODULES {
        name.hash(&mut hasher);
        src.hash(&mut hasher);
    }
    let h = hasher.finish();

    let mut dir = std::env::temp_dir();
    dir.push(format!("mamba_vendor_lib_{h:016x}"));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }

    for (name, src) in VENDORED_MODULES {
        let file = dir.join(format!("{name}.py"));
        // Only (re)write if missing or stale; ignore write races between processes.
        let needs_write = match std::fs::read_to_string(&file) {
            Ok(existing) => existing != *src,
            Err(_) => true,
        };
        if needs_write {
            // Write to a unique temp name then rename, so a partially written
            // file is never observed by a concurrent reader.
            let tmp = dir.join(format!("{name}.{}.tmp", std::process::id()));
            if let Ok(mut f) = std::fs::File::create(&tmp) {
                if f.write_all(src.as_bytes()).is_ok() {
                    let _ = std::fs::rename(&tmp, &file);
                }
            }
        }
    }

    Some(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_materialize_vendor_tree_writes_all_modules() {
        let dir = materialize_vendor_tree().expect("vendor tree should materialize");
        for (name, src) in VENDORED_MODULES {
            let path = dir.join(format!("{name}.py"));
            assert!(path.exists(), "{name}.py should exist in the vendored tree");
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                *src,
                "{name}.py on disk should match the embedded source"
            );
        }
    }
}
