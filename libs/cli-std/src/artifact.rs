//! Shared rendering helpers for service CLI deployment artifacts.
//!
//! K8s-native service CLIs all render the same classes of byte artifacts:
//! checked-in Dockerfiles, operator manifests, and CRD/instance YAML. The
//! service owns its domain-specific body; this module owns the presentation
//! hygiene that must remain uniform across those CLIs.

use std::io;
use std::path::{Path, PathBuf};

/// Normalize an optional version into this service's release tag.
///
/// A blank value falls back to the compiled package version. Supplying an
/// already-prefixed value preserves it byte-for-byte.
pub fn release_tag(project: &str, version: Option<&str>, fallback_version: &str) -> String {
    let raw = version
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback_version)
        .trim();
    let prefix = format!("{project}@");
    if raw.starts_with(&prefix) {
        raw.to_string()
    } else {
        format!("{prefix}{raw}")
    }
}

/// Remove source-ownership markers from an artifact intended for users to
/// build or apply.
pub fn strip_source_ownership_markers(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("# SPEC-MANAGED:")
            || trimmed == "# CODEGEN-BEGIN"
            || trimmed == "# CODEGEN-END"
        {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Substitute the checked-in operator namespace in an operator manifest.
pub fn replace_kubernetes_namespace(
    input: &str,
    checked_in_namespace: &str,
    namespace: &str,
) -> String {
    input
        .replace(
            &format!("name: {checked_in_namespace}"),
            &format!("name: {namespace}"),
        )
        .replace(
            &format!("namespace: {checked_in_namespace}"),
            &format!("namespace: {namespace}"),
        )
}

/// Ensure a text artifact ends with a trailing newline.
pub fn ensure_trailing_newline(input: &str) -> String {
    if input.ends_with('\n') {
        input.to_string()
    } else {
        format!("{input}\n")
    }
}

/// Write an artifact to an explicit file or a named file under an output
/// directory. Without `out`, stream the artifact to stdout.
///
/// The returned path exists only when bytes were written to disk, so a caller
/// can append a service-specific chainable `next:` line without duplicating
/// output-path resolution and file creation.
pub fn write_or_print(
    out: Option<&Path>,
    default_file: &str,
    body: &str,
) -> io::Result<Option<PathBuf>> {
    let Some(path) = out else {
        print!("{body}");
        return Ok(None);
    };

    let target = if path.extension().is_some() {
        path.to_path_buf()
    } else {
        path.join(default_file)
    };
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&target, body)?;
    println!("wrote {}", target.display());
    Ok(Some(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_tag_accepts_bare_prefixed_and_default_versions() {
        assert_eq!(release_tag("tape", Some("1.2.3"), "0.0.0"), "tape@1.2.3");
        assert_eq!(
            release_tag("tape", Some("tape@1.2.3"), "0.0.0"),
            "tape@1.2.3"
        );
        assert_eq!(release_tag("tape", None, "0.4.7"), "tape@0.4.7");
    }

    #[test]
    fn render_hygiene_strips_markers_and_replaces_only_namespace_fields() {
        let input = "# SPEC-MANAGED: source\n# CODEGEN-BEGIN\nkind: Namespace\nmetadata:\n  name: tape-system\n---\nmetadata:\n  namespace: tape-system\n# CODEGEN-END\n";
        let rendered = replace_kubernetes_namespace(
            &strip_source_ownership_markers(input),
            "tape-system",
            "staging",
        );
        assert_eq!(
            rendered,
            "kind: Namespace\nmetadata:\n  name: staging\n---\nmetadata:\n  namespace: staging\n"
        );
    }

    #[test]
    fn trailing_newline_is_preserved_or_added() {
        assert_eq!(ensure_trailing_newline("body"), "body\n");
        assert_eq!(ensure_trailing_newline("body\n"), "body\n");
    }

    #[test]
    fn write_or_print_resolves_a_directory_to_the_default_file() {
        let root = std::env::temp_dir().join(format!(
            "cli-std-artifact-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let target = write_or_print(Some(&root), "artifact.yaml", "body\n")
            .unwrap()
            .expect("disk output returns the written path");
        assert_eq!(target, root.join("artifact.yaml"));
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "body\n");
        std::fs::remove_dir_all(root).unwrap();
    }
}
