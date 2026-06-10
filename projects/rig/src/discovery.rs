//! Scenario discovery: walk a directory for `*.toml` scenario files,
//! parse, and lint. Pins/config trees are excluded by convention —
//! discovery only descends `scenarios/`-rooted paths when present.

use std::path::{Path, PathBuf};

use crate::scenario::{parse_scenario, LintViolation, Scenario};

/// One discovered file: parsed scenario or its lint violations.
pub struct Discovered {
    pub path: PathBuf,
    pub result: Result<Scenario, Vec<LintViolation>>,
}

/// Recursively collect every `*.toml` under `root` (sorted for
/// determinism), skipping `config/` and dot-directories.
pub fn discover(root: &Path) -> std::io::Result<Vec<Discovered>> {
    let mut files = Vec::new();
    collect_toml_files(root, &mut files)?;
    files.sort();
    Ok(files
        .into_iter()
        .map(|path| {
            let result = match std::fs::read_to_string(&path) {
                Ok(text) => parse_scenario(&path, &text),
                Err(e) => Err(vec![LintViolation {
                    message: format!("unreadable: {e}"),
                }]),
            };
            Discovered { path, result }
        })
        .collect())
}

fn collect_toml_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if name.starts_with('.') || name == "config" {
                continue;
            }
            collect_toml_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discovers_and_lints_a_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let dim = tmp.path().join("resilience");
        fs::create_dir_all(&dim).unwrap();
        fs::write(
            dim.join("ok_case.toml"),
            r#"
[record]
suite = "demo"
dimension = "resilience"
case = "ok_case"
subject = "demo"
kind = "e2e"
expected = "pass"

[[steps]]
type = "sleep"
name = "settle"
secs = 1
"#,
        )
        .unwrap();
        fs::write(
            dim.join("bad_case.toml"),
            r#"
[record]
suite = "demo"
dimension = "WRONG"
case = "mismatch"
subject = "demo"
kind = "e2e"
expected = "pass"

[[steps]]
type = "sleep"
name = "settle"
secs = 1
"#,
        )
        .unwrap();
        // config/ must be skipped.
        let cfg = tmp.path().join("config");
        fs::create_dir_all(&cfg).unwrap();
        fs::write(cfg.join("pins.toml"), "[[pins]]\n").unwrap();

        let found = discover(tmp.path()).unwrap();
        assert_eq!(found.len(), 2);
        let ok = found.iter().find(|d| d.path.ends_with("ok_case.toml")).unwrap();
        assert!(ok.result.is_ok());
        let bad = found.iter().find(|d| d.path.ends_with("bad_case.toml")).unwrap();
        let v = bad.result.as_ref().unwrap_err();
        assert!(v.len() >= 2); // case stem + dimension mismatches
    }
}
