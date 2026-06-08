//! Real `plistlib` for Mamba.
//!
//! plistlib is large, stateful, and full of rich-dunder value types (the `UID`
//! class, the `InvalidFileException` hierarchy, the `PlistFormat` enum-likes)
//! plus two complete serialization formats (Apple XML and binary `bplist00`).
//! Re-implementing all of that as native Rust dispatchers would mean
//! special-casing every dunder in `class.rs`. Instead we ship a pure-Python
//! port (`py_src/plistlib.py`) and let Mamba's own compiler execute it: the
//! module is materialized to a per-build temp directory at startup and that
//! directory is added to the import search path so `import plistlib` resolves
//! to the real implementation.
//!
//! The old long_tail stub (which returned empty strings/dicts from every
//! dump/load call) is removed in favour of this module.

use std::io::Write;
use std::path::PathBuf;

/// The pure-Python plistlib source, embedded at compile time.
const PLISTLIB_SRC: &str = include_str!("py_src/plistlib.py");

pub fn register() {
    // Materialize the embedded source to a stable temp directory and add that
    // directory to the import search path. Keyed on a content hash so the file
    // is written at most once per build and concurrent runs share it.
    let dir = match materialize_py_src() {
        Some(d) => d,
        None => return,
    };
    // Inserting at index 0 is always a valid Vec position. A user-supplied
    // plistlib.py in the running script's directory still wins regardless of
    // ordering: find_module() consults SCRIPT_DIR before SEARCH_PATHS.
    super::super::module::mb_insert_search_path(0, &dir.display().to_string());
}

fn materialize_py_src() -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    PLISTLIB_SRC.hash(&mut hasher);
    let h = hasher.finish();

    let mut dir = std::env::temp_dir();
    dir.push(format!("mamba_pylib_{h:016x}"));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let file = dir.join("plistlib.py");
    // Only (re)write if missing or stale; ignore write races between processes.
    let needs_write = match std::fs::read_to_string(&file) {
        Ok(existing) => existing != PLISTLIB_SRC,
        Err(_) => true,
    };
    if needs_write {
        // Write to a unique temp name then rename, so a partially written file
        // is never observed by a concurrent reader.
        let tmp = dir.join(format!("plistlib.{}.tmp", std::process::id()));
        if let Ok(mut f) = std::fs::File::create(&tmp) {
            if f.write_all(PLISTLIB_SRC.as_bytes()).is_ok() {
                let _ = std::fs::rename(&tmp, &file);
            }
        }
    }
    Some(dir)
}
