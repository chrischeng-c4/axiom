// HANDWRITE-BEGIN gap="missing-generator:hand-written:44214df1" tracker="enhancement-sdd-codegen-canonical-fingerprint" reason="Generator does not yet emit canonicalisation routines from a YAML schema."

//! `pyproject.toml` canonical fingerprint compute (R4).
//!
//! See `.aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic` —
//! the `lockfile-input-hash` flowchart drives the canonicalisation.

use std::path::Path;

use sha2::{Digest, Sha256};

use super::LockfileError;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/lockfile.md#logic (lockfile-input-hash)
pub(super) fn compute(pyproject_path: &Path) -> Result<String, LockfileError> {
    let text = std::fs::read_to_string(pyproject_path)
        .map_err(|e| LockfileError::Io { detail: e.to_string() })?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|e| LockfileError::TomlDecode { detail: e.to_string() })?;

    let project = value.get("project").and_then(|v| v.as_table());

    let mut deps: Vec<String> = project
        .and_then(|t| t.get("dependencies"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    deps.sort();

    let mut canonical = String::new();
    for dep in &deps {
        canonical.push_str(dep);
        canonical.push('\n');
    }

    let optional = project
        .and_then(|t| t.get("optional-dependencies"))
        .and_then(|v| v.as_table());
    if let Some(table) = optional {
        let mut groups: Vec<&String> = table.keys().collect();
        groups.sort();
        for group in groups {
            canonical.push_str(&format!("[optional:{}]\n", group));
            let mut group_deps: Vec<String> = table
                .get(group)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            group_deps.sort();
            for dep in &group_deps {
                canonical.push_str(dep);
                canonical.push('\n');
            }
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    Ok(hex_encode(&digest))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

// HANDWRITE-END
