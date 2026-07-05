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
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root")
        .to_path_buf()
}

const ALLOWED_ROOT_DOCS: &[&str] = &[
    "README.md",
    "CONTRIBUTING.md",
    "CLAUDE.md",
    "AGENTS.md",
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
// CODEGEN-END
