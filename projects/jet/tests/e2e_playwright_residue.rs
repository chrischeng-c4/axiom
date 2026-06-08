// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! @spec .aw/changes/enhancement-remove-playwright-test-dependency-from-e2e-harness/specs/enhancement-remove-playwright-test-dependency-from-e2e-harness-spec.md#R8

use std::fs;
use std::path::Path;

// REQ: R1
// REQ: R8
#[test]
fn e2e_playwright_residue_absent() {
    // Walk e2e/ recursively; assert no .ts file contains '@playwright/test'
    let repo_root = std::env::current_dir()
        .expect("cwd")
        .ancestors()
        .find(|p| p.join("Cargo.toml").is_file() && p.join("e2e").is_dir())
        .expect("find repo root")
        .to_path_buf();

    let e2e = repo_root.join("e2e");
    let mut offenders: Vec<String> = Vec::new();
    walk(&e2e, &mut offenders);
    assert!(
        offenders.is_empty(),
        "Found @playwright/test residue:\n{}",
        offenders.join("\n")
    );
}

fn walk(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|s| s.to_str()) == Some("node_modules") {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (lineno, line) in content.lines().enumerate() {
                        if line.contains("@playwright/test") {
                            out.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                lineno + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
    }
}
// CODEGEN-END
