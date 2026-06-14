//! Corpus round-trip proof for the rust-source-unit IR.
//!
//! Walks the crate's own `src/` tree — thousands of real, hand-written and
//! generated Rust files — and asserts that for every file `parse` accepts,
//! `emit` reproduces it byte-for-byte. This is the load-bearing invariant
//! behind moving Rust units off source-replay: an unchanged item-tree must
//! regenerate identical bytes, on real code, not just hand-picked snippets.
//!
//! @spec projects/agentic-workflow/tech-design/validate/rust-source-unit-ir-lossless-cst-parse-to-structured-item-tree-b.md#logic

use agentic_workflow::generate::rust_source_unit::parse;
use std::path::{Path, PathBuf};

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn entire_src_tree_round_trips_byte_identical() {
    let mut files = Vec::new();
    collect_rs_files(&src_root(), &mut files);
    assert!(
        files.len() > 100,
        "expected a substantial corpus, found {}",
        files.len()
    );

    let mut parsed = 0usize;
    let mut rejected = 0usize;
    let mut mismatches: Vec<String> = Vec::new();

    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        match parse(&src) {
            Ok(unit) => {
                parsed += 1;
                if unit.emit() != src {
                    mismatches.push(path.display().to_string());
                }
            }
            // A real source tree may contain a fixture or in-progress file that
            // does not parse clean; the contract only promises byte-identity for
            // units we accept. Such files are counted, not failed.
            Err(_) => rejected += 1,
        }
    }

    eprintln!(
        "rust-source-unit corpus: {} files, {} parsed clean, {} rejected, {} byte-mismatches",
        files.len(),
        parsed,
        rejected,
        mismatches.len()
    );

    assert!(
        mismatches.is_empty(),
        "byte-identical emit must hold for every accepted unit; mismatched: {:#?}",
        mismatches
    );
    // The overwhelming majority of our own tree must parse clean, else the
    // primitive is too weak to carry the td_ast migration.
    assert!(
        parsed * 100 >= files.len() * 95,
        "expected >=95% of the tree to parse clean, got {}/{}",
        parsed,
        files.len()
    );
}
