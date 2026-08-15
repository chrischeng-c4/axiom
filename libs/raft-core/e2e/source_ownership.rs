//! Source ownership and specification header integrity (#3592).
//!
//! Verifies that every `SPEC-MANAGED` header in `libs/raft-core` points to a
//! path that actually exists in the repository. A dangling header pointing to a
//! non-existent path fails the test and reports the file and line number.

use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().and_then(|p| p.parent()).expect(
        "CARGO_MANIFEST_DIR must have at least two parent directories to reach repository root",
    );
    let check_path = repo_root.join("libs/raft-core/Cargo.toml");
    assert!(
        check_path.is_file(),
        "Failed to resolve repository root from CARGO_MANIFEST_DIR ({}): expected {} to exist",
        manifest_dir.display(),
        check_path.display()
    );
    repo_root.to_path_buf()
}

fn collect_rs_files(dir: &Path, acc: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => panic!("Failed to read directory {}: {}", dir.display(), err),
    };
    let mut entries: Vec<_> = entries
        .map(|res| res.expect("Failed to read directory entry"))
        .collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, acc);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            acc.push(path);
        }
    }
}

#[test]
fn spec_managed_headers_name_existing_paths() {
    let repo_root = repository_root();
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut rs_files = Vec::new();
    collect_rs_files(crate_dir, &mut rs_files);
    assert!(
        !rs_files.is_empty(),
        "Expected to find .rs files in {}",
        crate_dir.display()
    );

    let mut dangling = Vec::new();

    for file_path in &rs_files {
        let content = std::fs::read_to_string(file_path)
            .unwrap_or_else(|err| panic!("Failed to read {}: {}", file_path.display(), err));
        let rel_path = file_path.strip_prefix(&repo_root).unwrap_or(file_path);

        for (idx, line) in content.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("// SPEC-MANAGED:") {
                let spec_target = rest.trim().split_whitespace().next().unwrap_or("");
                let file_target = spec_target.split('#').next().unwrap_or("");
                if !file_target.is_empty() {
                    let resolved = repo_root.join(file_target);
                    if !resolved.exists() {
                        dangling.push((rel_path.to_path_buf(), line_no, file_target.to_string()));
                    }
                }
            }
        }
    }

    if !dangling.is_empty() {
        let mut message = format!(
            "Found {} dangling SPEC-MANAGED header(s) in libs/raft-core:\n",
            dangling.len()
        );
        for (file, line, target) in &dangling {
            message.push_str(&format!(
                "  {}: line {}: SPEC-MANAGED header points to non-existent path '{}'\n",
                file.display(),
                line,
                target
            ));
        }
        panic!("{}", message);
    }
}
