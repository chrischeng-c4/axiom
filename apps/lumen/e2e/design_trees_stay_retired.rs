//! The sixteen projects the TD/EC retirement emptied must stay empty.
//!
//! S4 of that campaign deleted thirty trees — a `tech-design/` and an
//! `external-contracts/` under each of the fifteen projects USER DECISION D1
//! named. Nothing else refuses their return. The producer that wrote them (the
//! `aw` CLI) is deleted, so a tree that reappears here arrives by hand, and a
//! `@spec` line that survives points at a design document with no owner and no
//! generator behind it. `apps/tape` joined the list on 2026-08-26, when its
//! own two trees were deleted the same way.
//!
//! Two properties, and they fail differently on purpose:
//!
//! 1. **No tree.** A directory named `tech-design` or `external-contracts` at
//!    any depth under a retired project. This is structural — it does not read a
//!    single byte of any file, so no amount of prose can talk it out of a
//!    verdict.
//! 2. **No pointer.** A line carrying `@spec`. That marker is the retired
//!    mechanism's own syntax: it named the design document a file was generated
//!    from. Prose that merely *mentions* `tech-design/` is left alone — several
//!    files in these projects say the tree is gone, and saying so is not the
//!    regression.
//!
//! The third case is the one this campaign learned the hard way. Four separate
//! gates in it (hazards 29, 30, 31 and 33 of `docs/td-ec-retirement.md`) were
//! green because their scan was narrower than their declaration — an extension
//! whitelist, a literal-prefix match, a `cargo build` that never compiled the
//! target holding the break. So `the_sweep_is_not_vacuous` measures the
//! instrument rather than the tree: every project root must resolve, every
//! exemption must resolve, and the walk must reach a file count that a typo in
//! the list could not produce.
//!
//! Scope is sixteen projects and not the repository. That is a measurement, not
//! a preference: at the commit that introduced this file, 560 design-tree files
//! were still tracked across 26 other owners — `apps/tape` (since retired), `apps/pgpool`,
//! `apps/jet` and the rest — none of which D1 authorised anyone to touch. A
//! repository-wide assertion here would be red on its first run and would stay
//! red, which is a gate nobody can act on.

use std::fs;
use std::path::{Path, PathBuf};

/// The projects USER DECISION D1 named, plus `apps/tape` (retired 2026-08-26).
/// Each had both trees; each now has neither.
const RETIRED: [&str; 16] = [
    "apps/lumen",
    "apps/tape",
    "libs/build-stamp",
    "libs/cli-std",
    "libs/metrics-prometheus",
    "libs/openapi-codegen",
    "libs/peer-tls",
    "libs/raft-core",
    "libs/raft-runtime",
    "libs/service-auth",
    "libs/service-backup",
    "libs/service-http",
    "libs/service-k8s",
    "libs/service-observability",
    "libs/storage-durable",
    "libs/transport-h2c",
];

/// Directory names the retirement removed.
const DESIGN_DIRS: [&str; 2] = ["tech-design", "external-contracts"];

/// The retired mechanism's own marker: it named the design document a file was
/// generated from.
const POINTER: &str = "@spec";

/// Files whose job is to name the retired mechanism. Both are asserted to exist,
/// so renaming one fails this suite instead of silently widening the exemption.
const EXEMPT: [&str; 2] = [
    "apps/lumen/docs/td-ec-retirement.md",
    "apps/lumen/e2e/design_trees_stay_retired.rs",
];

/// Tracked files under the fifteen at the commit that introduced this file: 593;
/// under the sixteen when `apps/tape` joined: 663.
/// The floor is set well below that so ordinary growth and deletion do not touch
/// it, and well above what a mistyped project root could reach.
const MIN_FILES_SWEPT: usize = 400;

fn repo_root() -> PathBuf {
    // `apps/lumen` -> `apps` -> repository root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/lumen lives two levels below the repository root")
        .to_path_buf()
}

/// Walk one project, collecting its files and any design directory found on the
/// way. Build output and VCS metadata are skipped: a gate that reads `target/`
/// measures the compiler, not the repository.
fn walk(dir: &Path, files: &mut Vec<PathBuf>, design: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(name.as_ref(), "target" | ".git" | "node_modules") {
                continue;
            }
            if DESIGN_DIRS.contains(&name.as_ref()) {
                design.push(path.clone());
            }
            walk(&path, files, design);
        } else {
            files.push(path);
        }
    }
}

fn sweep(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let (mut files, mut design) = (Vec::new(), Vec::new());
    for project in RETIRED {
        walk(&root.join(project), &mut files, &mut design);
    }
    files.sort();
    design.sort();
    (files, design)
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn no_retired_project_carries_a_design_tree() {
    let root = repo_root();
    let (_, design) = sweep(&root);
    assert!(
        design.is_empty(),
        "the TD/EC retirement deleted every design tree under these projects; \
         {} has come back: {:?}\n\
         The `.rs` file is the authoring surface — a module's `//!` block carries \
         the rules it owns. See CLAUDE.md \"Artifact write order\".",
        design.len(),
        design
            .iter()
            .map(|p| relative(&root, p))
            .collect::<Vec<_>>()
    );
}

#[test]
fn no_retired_project_carries_a_spec_pointer() {
    let root = repo_root();
    let (files, _) = sweep(&root);
    let exempt: Vec<PathBuf> = EXEMPT.iter().map(|p| root.join(p)).collect();

    let mut hits = Vec::new();
    for file in &files {
        if exempt.contains(file) {
            continue;
        }
        let Ok(bytes) = fs::read(file) else {
            continue;
        };
        // Binary content has nothing to say about a doc-comment marker, and
        // decoding it lossily would invent characters that are not in the file.
        if bytes.contains(&0) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        for (line_no, line) in text.lines().enumerate() {
            if line.contains(POINTER) {
                hits.push(format!(
                    "{}:{}: {}",
                    relative(&root, file),
                    line_no + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "`{POINTER}` is the retired mechanism's own marker: it names the design \
         document a file was generated from. Its producer is deleted, so each of \
         these {} lines points at a document nothing maintains.\n{}\n\
         Delete the line. What it was worth keeping belongs in the surrounding \
         `//!` or `///` block, which is the authoring surface now.",
        hits.len(),
        hits.join("\n")
    );
}

#[test]
fn the_sweep_is_not_vacuous() {
    let root = repo_root();

    for project in RETIRED {
        let dir = root.join(project);
        assert!(
            dir.is_dir(),
            "the sweep list names `{project}`, which is not a directory; \
             a stale entry sweeps nothing while still reading as coverage"
        );
    }

    for path in EXEMPT {
        assert!(
            root.join(path).is_file(),
            "the exemption list names `{path}`, which is not a file; \
             an exemption that resolves to nothing is a hole nobody is watching"
        );
    }

    let (files, _) = sweep(&root);
    assert!(
        files.len() >= MIN_FILES_SWEPT,
        "the sweep reached {} files, below the floor of {MIN_FILES_SWEPT}; \
         a sweep that visits nothing passes every assertion above having read \
         no repository at all",
        files.len()
    );
}
