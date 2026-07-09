// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! `init` verb — scaffold the three parity files into a target dir.

use std::path::{Path, PathBuf};

use crate::manifest::GateError;

const MANIFEST_TEMPLATE: &str = include_str!("../templates/parity-gating.toml");
const WAIVERS_TEMPLATE: &str = include_str!("../templates/waivers.toml");
const DOCS_TEMPLATE: &str = include_str!("../templates/gating-manifest.md");

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Clone)]
pub struct InitReport {
    pub written: Vec<PathBuf>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
pub fn run_init(target_dir: &Path, force: bool) -> Result<InitReport, GateError> {
    std::fs::create_dir_all(target_dir).map_err(|e| GateError::Io {
        path: target_dir.display().to_string(),
        source: e,
    })?;
    let docs_dir = target_dir.join("docs");
    std::fs::create_dir_all(&docs_dir).map_err(|e| GateError::Io {
        path: docs_dir.display().to_string(),
        source: e,
    })?;

    let manifest_path = target_dir.join("parity-gating.toml");
    let waivers_path = target_dir.join("waivers.toml");
    let docs_path = docs_dir.join("gating-manifest.md");

    let mut written = Vec::new();
    for (path, content) in [
        (&manifest_path, MANIFEST_TEMPLATE),
        (&waivers_path, WAIVERS_TEMPLATE),
        (&docs_path, DOCS_TEMPLATE),
    ] {
        if path.exists() && !force {
            return Err(GateError::WouldOverwrite {
                path: path.display().to_string(),
            });
        }
        std::fs::write(path, content).map_err(|e| GateError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        written.push(path.clone());
    }

    Ok(InitReport { written })
}
// CODEGEN-END
