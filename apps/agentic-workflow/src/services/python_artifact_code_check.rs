// HANDWRITE-BEGIN gap="missing-generator:python-artifact-code-check" tracker="#2305" reason="The terminal graph verifier composes compiler and target manifests until the Python artifact protocol generator owns the closure."
//! Cold target verification for the Python-v1 terminal artifact graph.

use super::{python_td::compile_python_td_project, python_td_target::emit_python_td_target};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTargetBuildCheck {
    pub td_semantic_digest: String,
    pub target_build_digest: String,
    pub clean: bool,
    pub drifted_paths: Vec<String>,
}

/// Compile the TD into a fresh target directory, then compare only the files
/// owned by the emitter manifest. Unrelated product files are intentionally
/// outside this comparison and cannot create either a false red or a false
/// green for generated output.
pub fn verify_python_target_build(td_root: &Path, output_root: &Path) -> Result<PythonTargetBuildCheck> {
    let ir = compile_python_td_project(td_root)?;
    let cold = tempfile::tempdir().context("create Python TD cold output directory")?;
    let target = emit_python_td_target(&ir, cold.path())?;
    let mut drifted_paths = Vec::new();
    for file in &target.files {
        let expected = cold.path().join(&file.path);
        let actual = output_root.join(&file.path);
        let matches = match (fs::read(&expected), fs::read(&actual)) {
            (Ok(expected), Ok(actual)) => expected == actual,
            _ => false,
        };
        if !matches {
            drifted_paths.push(file.path.clone());
        }
    }
    drifted_paths.sort();
    Ok(PythonTargetBuildCheck {
        td_semantic_digest: ir.semantic_digest,
        target_build_digest: target.digest,
        clean: drifted_paths.is_empty(),
        drifted_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_python_target_build_detects_only_manifest_owned_drift() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\ndef issue_invoice() -> None:\n    pass\n",
        )
        .unwrap();
        let ir = compile_python_td_project(td.path()).unwrap();
        emit_python_td_target(&ir, output.path()).unwrap();
        fs::write(output.path().join("notes.txt"), "user-owned\n").unwrap();

        let clean = verify_python_target_build(td.path(), output.path()).unwrap();
        assert!(clean.clean, "{clean:#?}");
        assert!(clean.drifted_paths.is_empty());

        fs::write(output.path().join("src/demo/domain/invoice.py"), "changed\n").unwrap();
        let drifted = verify_python_target_build(td.path(), output.path()).unwrap();
        assert!(!drifted.clean);
        assert_eq!(drifted.drifted_paths, vec!["src/demo/domain/invoice.py"]);
    }
}
// HANDWRITE-END
