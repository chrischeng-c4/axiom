// HANDWRITE-BEGIN gap="missing-generator:meta-runtime-adapter-test" tracker="#1816" reason="Runtime adapter topology is not yet a generated test primitive"
//! Root runtime-adapter contract.
//!
//! `AGENTS.md` is the compact shared authority and generated Codex rule index.
//! `CLAUDE.md` imports it and owns only Claude-specific loading behavior.
//! Shared reusable facts live in `.agents/rules`, not in a mirrored pair of
//! always-loaded root documents.

use agentic_workflow::cli::meta_docs::{meta_doc_contract, MetaDocLayer};
use agentic_workflow::cli::meta_schema::{RULE_INDEX_END, RULE_INDEX_START};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn claude_is_delta_over_agents_and_agents_routes_canonical_rules() {
    for filename in ["AGENTS.md", "CLAUDE.md"] {
        assert!(
            meta_doc_contract(MetaDocLayer::Repository, filename).is_some(),
            "{filename} must be owned by the repository layer"
        );
        assert!(
            meta_doc_contract(MetaDocLayer::Project, filename).is_none(),
            "{filename} must not acquire a project-layer owner"
        );
    }

    let root = repo_root();
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    let claude = fs::read_to_string(root.join("CLAUDE.md")).expect("read CLAUDE.md");

    assert!(agents.contains("## Agentic Workflow CLI Surface"));
    assert!(agents.contains(RULE_INDEX_START));
    assert!(agents.contains(RULE_INDEX_END));
    assert!(agents.contains(".agents/rules"));
    assert!(agents.lines().count() < 180, "AGENTS.md must stay a compact bootstrap");

    assert!(claude.lines().any(|line| line.trim() == "@AGENTS.md"));
    assert!(claude.contains("## Claude Runtime Adapter"));
    assert!(!claude.contains("## Agentic Workflow CLI Surface"));
    assert!(
        claude.lines().count() < 60,
        "CLAUDE.md must contain only the import and Claude-specific delta"
    );
}
// HANDWRITE-END
