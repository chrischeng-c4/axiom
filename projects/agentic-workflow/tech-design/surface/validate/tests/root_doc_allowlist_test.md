---
id: projects-agentic-workflow-tests-cli-tests-root-doc-allowlist-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: partial
    rationale: "Repo-root doc allowlist contract (meta-doc sheet 2): executable guarantee that the repo root carries only the five allowed docs, replacing manual keep-the-root-clean discipline for stray project/session docs."
---

# Standardized projects/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs`.

### Symbols

No public AST symbols.

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs -->
```rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/root_doc_allowlist_test.md#source
// CODEGEN-BEGIN
//! Repo-root doc allowlist contract (meta-doc sheet 2).
//!
//! The repo root carries exactly four visible `.md` files: `README.md`
//! (project and shared-library inventory), `CONTRIBUTING.md` (repo-wide
//! authoring contract), and `CLAUDE.md` / `AGENTS.md` (the implementation
//! quick-reference in its two agent-runtime flavors).
//! Every other root-level doc fact needs a generator, a validator, or a
//! policy-only home under `projects/<p>/` or next to the tree it governs —
//! see the meta-doc content contract in `CONTRIBUTING.md`. This test
//! replaces "don't drop stray docs at the root" tribal knowledge with an
//! executable contract (wave 1 of the doc consolidation already retired the
//! prior strays: `QUICKSTART.md`, `TESTING.md`, `CHANGELOG.md`, `GOAL.md`).
//!
//! Dotfiles (e.g. tool-specific prompt/cache files such as
//! `.jet.loop-prompt.md`) are intentionally out of scope: they are not
//! documentation and shell `*.md` globbing does not surface them either.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root")
        .to_path_buf()
}

const ALLOWED_ROOT_DOCS: &[&str] = &["README.md", "CONTRIBUTING.md", "CLAUDE.md", "AGENTS.md"];
const ALLOWED_ROOT_UPPERCASE_META: &[&str] = &[
    "README.md",
    "CONTRIBUTING.md",
    "CLAUDE.md",
    "AGENTS.md",
    "LICENSE",
];

#[test]
fn repo_root_md_files_match_allowlist() {
    let root = repo_root();
    let allowed: BTreeSet<&str> = ALLOWED_ROOT_DOCS.iter().copied().collect();

    let mut found: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(&root).expect("read repo root") {
        let entry = entry.expect("read repo root entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if name.ends_with(".md") {
            found.insert(name.to_string());
        }
    }

    let unexpected: Vec<&String> = found
        .iter()
        .filter(|name| !allowed.contains(name.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "unexpected root-level doc(s) {unexpected:?} — repo root carries only \
         {ALLOWED_ROOT_DOCS:?}; see CONTRIBUTING.md meta-doc content contract \
         and move the content: project docs live under projects/<p>/, \
         conventions live next to the tree they govern"
    );

    let missing: Vec<&&str> = ALLOWED_ROOT_DOCS
        .iter()
        .filter(|name| !found.contains(&***name))
        .collect();
    assert!(
        missing.is_empty(),
        "expected root doc(s) missing from repo root: {missing:?}"
    );
}

fn is_uppercase_meta_filename(name: &str) -> bool {
    if name == "LICENSE" {
        return true;
    }
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let mut chars = stem.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase()
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
}

#[test]
fn repo_root_uppercase_meta_files_match_allowlist() {
    let root = repo_root();
    let allowed: BTreeSet<&str> = ALLOWED_ROOT_UPPERCASE_META.iter().copied().collect();

    let mut found: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(&root).expect("read repo root") {
        let entry = entry.expect("read repo root entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if is_uppercase_meta_filename(name) {
            found.insert(name.to_string());
        }
    }

    let unexpected: Vec<&String> = found
        .iter()
        .filter(|name| !allowed.contains(name.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "unexpected root uppercase meta file(s) {unexpected:?} — root allows only \
         {ALLOWED_ROOT_UPPERCASE_META:?}; LICENSE is the only uppercase meta file \
         without a .md extension"
    );
}

fn collect_live_agent_docs(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("read {}: {err}", dir.display());
    });
    for entry in entries {
        let entry = entry.expect("read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_live_agent_docs(root, &path, out);
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !matches!(name, "CLAUDE.md" | "AGENTS.md") {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        if rel == Path::new("projects/agentic-workflow/templates/cli/mainthread/CLAUDE.md") {
            continue;
        }
        out.push(rel.to_path_buf());
    }
}

#[test]
fn project_layer_does_not_define_live_agent_docs() {
    let root = repo_root();
    let mut found = Vec::new();
    collect_live_agent_docs(&root, &root.join("projects"), &mut found);
    assert!(
        found.is_empty(),
        "project-layer CLAUDE.md/AGENTS.md files are not allowed; these names \
         belong to the repo/global layer only. Move project-specific agent \
         facts into README/CONTRIBUTING/CAPABILITIES, scoped convention docs, \
         skills/templates, or command output. Unexpected files: {found:?}"
    );
}

fn markdown_section_body(doc: &str, heading: &str) -> String {
    let heading_marker = format!("\n## {heading}\n");
    let Some(start) = doc
        .find(&heading_marker)
        .map(|index| index + heading_marker.len())
    else {
        return String::new();
    };
    let tail = &doc[start..];
    let end = tail.find("\n## ").unwrap_or(tail.len());
    tail[..end].trim().to_string()
}

#[test]
fn agentic_workflow_readme_links_project_layer_meta_docs() {
    let root = repo_root();
    let readme_path = root.join("projects/agentic-workflow/README.md");
    let readme = fs::read_to_string(&readme_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", readme_path.display()));
    let contributing_path = root.join("projects/agentic-workflow/CONTRIBUTING.md");
    let contributing = fs::read_to_string(&contributing_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", contributing_path.display()));
    let capabilities_path = root.join("projects/agentic-workflow/CAPABILITIES.md");
    let capabilities = fs::read_to_string(&capabilities_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", capabilities_path.display()));
    let contributing_brief = markdown_section_body(&contributing, "Brief");
    let capabilities_brief = markdown_section_body(&capabilities, "Brief");
    let readme_contributing = markdown_section_body(&readme, "Contributing");
    let readme_capability_contract = markdown_section_body(&readme, "Capability Contract");
    assert!(
        !contributing_brief.is_empty(),
        "project-layer CONTRIBUTING.md must expose a `## Brief` section"
    );
    assert!(
        !capabilities_brief.is_empty(),
        "project-layer CAPABILITIES.md must expose a `## Brief` section"
    );
    assert!(
        !readme_contributing.is_empty(),
        "project README must expose a fixed `## Contributing` section"
    );
    assert!(
        readme_contributing.contains("[CONTRIBUTING.md](CONTRIBUTING.md)"),
        "project README `## Contributing` must point to project-layer CONTRIBUTING.md"
    );
    assert!(
        readme_contributing.contains(&contributing_brief),
        "project README `## Contributing` must include the project CONTRIBUTING brief"
    );
    assert!(
        !readme_capability_contract.is_empty(),
        "project README must expose a fixed `## Capability Contract` section"
    );
    assert!(
        readme_capability_contract.contains("[CAPABILITIES.md](CAPABILITIES.md)"),
        "project README `## Capability Contract` must point to project-layer CAPABILITIES.md"
    );
    assert!(
        readme_capability_contract.contains(&capabilities_brief),
        "project README `## Capability Contract` must include the project CAPABILITIES brief"
    );
}
// CODEGEN-END

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/root_doc_allowlist_test.rs
    action: create
    section: source
    description: |
      Repo-root doc allowlist contract test
      repo_root_md_files_match_allowlist (meta-doc sheet 2, doc consolidation
      wave 2): the repo root carries exactly README.md, CONTRIBUTING.md,
      CLAUDE.md, and AGENTS.md; any other visible root-level
      `.md` file fails the suite naming the unexpected file(s) and pointing
      at the CONTRIBUTING.md meta-doc content contract. Wave 1 of the doc
      consolidation already deleted the prior strays (QUICKSTART.md,
      TESTING.md, CHANGELOG.md, GOAL.md moved), so this test is green on
      introduction.
    impl_mode: hand-written
```
