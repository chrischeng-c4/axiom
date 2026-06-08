//! Real `fileinput` for Mamba.
//!
//! fileinput is a stateful module: it iterates lines across a list of files
//! (or stdin), tracks per-file and cumulative line numbers, supports inplace
//! rewriting, custom open hooks, and binary/text modes. The previous stub
//! returned empty lists / fixed sentinels from every helper and could not
//! provide a working `FileInput` class.
//!
//! Rather than special-casing the whole `FileInput` protocol in native Rust,
//! we ship a pure-Python port (`py_src/fileinput.py`, adapted from CPython
//! 3.12) and let Mamba's own compiler execute it: the source is materialized
//! to a per-build temp directory at startup and that directory is added to the
//! import search path so `import fileinput` resolves to the real
//! implementation. No native module is registered, so the search-path file is
//! the only `fileinput` Mamba sees.

use std::io::Write;
use std::path::PathBuf;

/// The pure-Python fileinput source, embedded at compile time.
const FILEINPUT_SRC: &str = include_str!("py_src/fileinput.py");

pub fn register() {
    // Materialize the embedded source to a stable temp directory and add that
    // directory to the import search path. Keyed on a content hash so the file
    // is written at most once per build and concurrent runs share it.
    let dir = match materialize_py_src() {
        Some(d) => d,
        None => return,
    };
    // Inserting at index 0 is always a valid Vec position. A user-supplied
    // fileinput.py in the running script's directory still wins regardless of
    // ordering: find_module() consults SCRIPT_DIR before SEARCH_PATHS.
    super::super::module::mb_insert_search_path(0, &dir.display().to_string());
}

fn materialize_py_src() -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    FILEINPUT_SRC.hash(&mut hasher);
    let h = hasher.finish();

    let mut dir = std::env::temp_dir();
    dir.push(format!("mamba_pylib_fileinput_{h:016x}"));
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let file = dir.join("fileinput.py");
    // Only (re)write if missing or stale; ignore write races between processes.
    let needs_write = match std::fs::read_to_string(&file) {
        Ok(existing) => existing != FILEINPUT_SRC,
        Err(_) => true,
    };
    if needs_write {
        // Write to a unique temp name then rename, so a partially written file
        // is never observed by a concurrent reader.
        let tmp = dir.join(format!("fileinput.{}.tmp", std::process::id()));
        if let Ok(mut f) = std::fs::File::create(&tmp) {
            if f.write_all(FILEINPUT_SRC.as_bytes()).is_ok() {
                let _ = std::fs::rename(&tmp, &file);
            }
        }
    }
    Some(dir)
}
