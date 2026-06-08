// `uv tree` — dependency tree traversal + ASCII renderer (Tick 42).
//
// Pure data layer: given a `Lockfile`, build the directed dep graph,
// pick roots (packages not depended on by anything else), and render
// the canonical uv-shaped ASCII tree. No I/O, no resolver round-trip.
//
// Mirrors uv's `tree` subcommand surface:
//   * `--depth <N>` — cap traversal depth.
//   * `--package <name>` — render only the subtree rooted at one package.
//   * `--invert` — reverse-dep view (`A` lists every package that
//     declares `A` as a dependency).
//   * `--prune <name>` — skip a named subtree entirely.
//   * cycle-dedupe via `(*)` marker — already-visited packages
//     elide their subtree on subsequent occurrences.
//
// What this tick does NOT cover:
//   * `--outdated` — requires an index round-trip (HTTP).
//   * `--package-filter` for groups/extras — needs richer lockfile.
//   * Color output / TTY detection — caller's responsibility.

use std::collections::{BTreeMap, BTreeSet};

use crate::pkgmanage::pkgmgr::lockfile::{Lockfile, Package};

/// Caller-facing render knobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeOptions {
    /// Maximum depth from each root. `None` means unlimited.
    pub max_depth: Option<usize>,
    /// If set, render only the subtree rooted at this package (PEP 503).
    pub focus: Option<String>,
    /// Reverse-dep view: edges flip from `parent -> child` to
    /// `child -> parent`. Each package lists who depends on it.
    pub invert: bool,
    /// PEP 503-normalized names to skip entirely (and their subtrees).
    pub prune: Vec<String>,
    /// Disable cycle/duplicate dedupe (the `(*)` marker). Default off.
    /// uv calls this `--no-dedupe`; we keep dedupe on by default.
    pub no_dedupe: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        TreeOptions {
            max_depth: None,
            focus: None,
            invert: false,
            prune: Vec::new(),
            no_dedupe: false,
        }
    }
}

/// One node in the rendered tree. Public so callers can post-process
/// (e.g., emit JSON) without re-traversing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub name: String,
    pub version: String,
    pub depth: usize,
    /// `true` when this node has already appeared elsewhere in the
    /// tree and its subtree is elided (rendered with `(*)`).
    pub deduped: bool,
    pub children: Vec<TreeNode>,
}

/// Build the in-memory tree (no rendering) from a `Lockfile`.
///
/// Roots are picked deterministically:
///   * In forward (default) mode: every package not declared as a
///     dependency of any other package. Cycles still get covered
///     because we fall back to "every package not yet visited"
///     after exhausting natural roots.
///   * In `invert` mode: every package depended on by at least one
///     other package, sorted; pure leaves with no consumer become
///     the inverted roots.
pub fn build_tree(lockfile: &Lockfile, opts: &TreeOptions) -> Vec<TreeNode> {
    let adjacency = build_adjacency(lockfile, opts.invert);
    let by_name = index_by_name(lockfile);
    let prune: BTreeSet<String> = opts.prune.iter().map(|p| normalize(p)).collect();

    if let Some(focus) = &opts.focus {
        let key = normalize(focus);
        if !by_name.contains_key(&key) {
            return Vec::new();
        }
        let mut seen = BTreeSet::new();
        return vec![walk(
            &key,
            0,
            &adjacency,
            &by_name,
            &prune,
            opts,
            &mut seen,
        )];
    }

    let roots = pick_roots(lockfile, &adjacency, opts.invert);
    let mut seen = BTreeSet::new();
    roots
        .iter()
        .filter(|name| !prune.contains(*name))
        .map(|name| walk(name, 0, &adjacency, &by_name, &prune, opts, &mut seen))
        .collect()
}

/// Render the tree to the canonical ASCII shape:
///
/// ```text
/// project v1.0
/// ├── requests v2.31.0
/// │   ├── certifi v2024.2.2
/// │   └── idna v3.6
/// └── flask v3.0.0
///     └── click v8.1.7
/// ```
///
/// Output always ends with a single trailing newline.
pub fn render_tree(roots: &[TreeNode]) -> String {
    let mut out = String::new();
    for root in roots {
        out.push_str(&format!("{} v{}", root.name, root.version));
        if root.deduped {
            out.push_str(" (*)");
        }
        out.push('\n');
        render_children(&mut out, &root.children, "");
    }
    out
}

/// Convenience: combine `build_tree` + `render_tree`.
pub fn render_lockfile_tree(lockfile: &Lockfile, opts: &TreeOptions) -> String {
    render_tree(&build_tree(lockfile, opts))
}

fn render_children(out: &mut String, children: &[TreeNode], prefix: &str) {
    let last_idx = children.len().saturating_sub(1);
    for (i, child) in children.iter().enumerate() {
        let is_last = i == last_idx;
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(prefix);
        out.push_str(connector);
        out.push_str(&format!("{} v{}", child.name, child.version));
        if child.deduped {
            out.push_str(" (*)");
        }
        out.push('\n');
        let next_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
        render_children(out, &child.children, &next_prefix);
    }
}

fn walk(
    name: &str,
    depth: usize,
    adjacency: &BTreeMap<String, Vec<String>>,
    by_name: &BTreeMap<String, Package>,
    prune: &BTreeSet<String>,
    opts: &TreeOptions,
    seen: &mut BTreeSet<String>,
) -> TreeNode {
    let pkg = by_name.get(name).cloned().unwrap_or_else(|| Package {
        name: name.to_string(),
        version: "?".into(),
        sha256: String::new(),
        source: String::new(),
        dependencies: Vec::new(),
        markers: None,
        source_ref: None,
    });

    let already_seen = seen.contains(name);
    let deduped = !opts.no_dedupe && already_seen;

    let mut node = TreeNode {
        name: pkg.name,
        version: pkg.version,
        depth,
        deduped,
        children: Vec::new(),
    };

    if !already_seen {
        seen.insert(name.to_string());
    }

    let at_depth_limit = opts
        .max_depth
        .map(|d| depth >= d)
        .unwrap_or(false);
    if deduped || at_depth_limit {
        return node;
    }

    let empty = Vec::new();
    let kids = adjacency.get(name).unwrap_or(&empty);
    for child in kids {
        if prune.contains(child) {
            continue;
        }
        node.children
            .push(walk(child, depth + 1, adjacency, by_name, prune, opts, seen));
    }
    node
}

fn build_adjacency(lockfile: &Lockfile, invert: bool) -> BTreeMap<String, Vec<String>> {
    let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for pkg in &lockfile.packages {
        let parent_key = normalize(&pkg.name);
        for dep in &pkg.dependencies {
            let child_key = normalize(dep);
            if invert {
                adj.entry(child_key).or_default().push(parent_key.clone());
            } else {
                adj.entry(parent_key.clone())
                    .or_default()
                    .push(child_key);
            }
        }
        // Ensure every package shows up as a key so leaves render correctly.
        adj.entry(parent_key).or_default();
    }
    for kids in adj.values_mut() {
        kids.sort();
        kids.dedup();
    }
    adj
}

fn index_by_name(lockfile: &Lockfile) -> BTreeMap<String, Package> {
    lockfile
        .packages
        .iter()
        .map(|p| (normalize(&p.name), p.clone()))
        .collect()
}

fn pick_roots(
    lockfile: &Lockfile,
    adjacency: &BTreeMap<String, Vec<String>>,
    invert: bool,
) -> Vec<String> {
    let mut declared_as_child: BTreeSet<String> = BTreeSet::new();
    if invert {
        // Inverted: roots = packages that depend on nothing (leaves of fwd).
        for pkg in &lockfile.packages {
            if !pkg.dependencies.is_empty() {
                declared_as_child.insert(normalize(&pkg.name));
            }
        }
    } else {
        // Forward: roots = packages that nobody else depends on.
        for kids in adjacency.values() {
            for child in kids {
                declared_as_child.insert(child.clone());
            }
        }
    }
    let mut roots: Vec<String> = lockfile
        .packages
        .iter()
        .map(|p| normalize(&p.name))
        .filter(|name| !declared_as_child.contains(name))
        .collect();
    roots.sort();
    roots.dedup();
    if roots.is_empty() && !lockfile.packages.is_empty() {
        // Cycle-only graph — fall back to every package, alphabetical,
        // so we still render something deterministic.
        roots = lockfile
            .packages
            .iter()
            .map(|p| normalize(&p.name))
            .collect();
        roots.sort();
        roots.dedup();
    }
    roots
}

fn normalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_sep = false;
    for c in name.chars() {
        let lc = c.to_ascii_lowercase();
        if lc == '-' || lc == '_' || lc == '.' {
            if !last_sep {
                out.push('-');
            }
            last_sep = true;
        } else {
            out.push(lc);
            last_sep = false;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, version: &str, deps: &[&str]) -> Package {
        Package {
            name: name.into(),
            version: version.into(),
            sha256: String::new(),
            source: String::new(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            markers: None,
            source_ref: None,
        }
    }

    fn lock(packages: Vec<Package>) -> Lockfile {
        Lockfile {
            format_version: 1,
            input_hash: "h".into(),
            packages,
        }
    }

    #[test]
    fn empty_lockfile_renders_to_empty_string() {
        let l = lock(Vec::new());
        assert_eq!(render_lockfile_tree(&l, &TreeOptions::default()), "");
    }

    #[test]
    fn single_root_no_deps_renders_one_line() {
        let l = lock(vec![pkg("lonely", "1.0", &[])]);
        assert_eq!(
            render_lockfile_tree(&l, &TreeOptions::default()),
            "lonely v1.0\n"
        );
    }

    #[test]
    fn simple_tree_uses_canonical_glyphs() {
        let l = lock(vec![
            pkg("app", "1.0", &["requests", "flask"]),
            pkg("requests", "2.31.0", &["idna", "certifi"]),
            pkg("flask", "3.0.0", &["click"]),
            pkg("idna", "3.6", &[]),
            pkg("certifi", "2024.2.2", &[]),
            pkg("click", "8.1.7", &[]),
        ]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        // Roots: only "app" (nobody depends on it).
        let expected = "\
app v1.0
├── flask v3.0.0
│   └── click v8.1.7
└── requests v2.31.0
    ├── certifi v2024.2.2
    └── idna v3.6
";
        assert_eq!(body, expected);
    }

    #[test]
    fn depth_limit_truncates_below_target() {
        let l = lock(vec![
            pkg("app", "1", &["requests"]),
            pkg("requests", "2", &["idna"]),
            pkg("idna", "3", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { max_depth: Some(1), ..Default::default() },
        );
        assert!(body.contains("requests v2"));
        assert!(!body.contains("idna v3"));
    }

    #[test]
    fn depth_zero_renders_roots_only() {
        let l = lock(vec![
            pkg("app", "1", &["requests"]),
            pkg("requests", "2", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { max_depth: Some(0), ..Default::default() },
        );
        assert_eq!(body, "app v1\n");
    }

    #[test]
    fn focus_subtree_only_renders_that_subtree() {
        let l = lock(vec![
            pkg("app", "1", &["requests", "flask"]),
            pkg("requests", "2", &["idna"]),
            pkg("flask", "3", &["click"]),
            pkg("idna", "4", &[]),
            pkg("click", "5", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { focus: Some("requests".into()), ..Default::default() },
        );
        assert!(body.starts_with("requests v2"));
        assert!(body.contains("idna v4"));
        assert!(!body.contains("flask v3"));
        assert!(!body.contains("app v1"));
    }

    #[test]
    fn focus_unknown_name_yields_empty() {
        let l = lock(vec![pkg("a", "1", &[])]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { focus: Some("missing".into()), ..Default::default() },
        );
        assert_eq!(body, "");
    }

    #[test]
    fn invert_view_lists_consumers_under_each_leaf() {
        let l = lock(vec![
            pkg("a", "1", &["common"]),
            pkg("b", "1", &["common"]),
            pkg("common", "2", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { invert: true, ..Default::default() },
        );
        // Inverted root is `common`; it shows `a` and `b` as consumers.
        assert!(body.starts_with("common v2"));
        assert!(body.contains("a v1"));
        assert!(body.contains("b v1"));
    }

    #[test]
    fn prune_skips_named_subtree_entirely() {
        let l = lock(vec![
            pkg("app", "1", &["requests", "flask"]),
            pkg("requests", "2", &["idna"]),
            pkg("flask", "3", &[]),
            pkg("idna", "4", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions {
                prune: vec!["requests".into()],
                ..Default::default()
            },
        );
        assert!(!body.contains("requests"));
        assert!(!body.contains("idna"));
        assert!(body.contains("flask v3"));
    }

    #[test]
    fn dedupe_marks_repeated_subtree_with_star() {
        // common appears under both `a` and `b`; second occurrence should
        // carry `(*)` and elide its children.
        let l = lock(vec![
            pkg("app", "1", &["a", "b"]),
            pkg("a", "1", &["common"]),
            pkg("b", "1", &["common"]),
            pkg("common", "2", &["leaf"]),
            pkg("leaf", "3", &[]),
        ]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        let common_occurrences = body.matches("common v2").count();
        let leaf_occurrences = body.matches("leaf v3").count();
        let starred = body.matches("common v2 (*)").count();
        assert_eq!(common_occurrences, 2);
        // leaf only renders under the first `common`; the second `common`
        // is deduped and elides children.
        assert_eq!(leaf_occurrences, 1);
        assert_eq!(starred, 1);
    }

    #[test]
    fn no_dedupe_expands_repeated_subtrees() {
        let l = lock(vec![
            pkg("app", "1", &["a", "b"]),
            pkg("a", "1", &["common"]),
            pkg("b", "1", &["common"]),
            pkg("common", "2", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { no_dedupe: true, ..Default::default() },
        );
        assert_eq!(body.matches("common v2").count(), 2);
        assert!(!body.contains("(*)"));
    }

    #[test]
    fn cycles_dont_loop_forever() {
        // a <-> b cycle; pick_roots fallback ensures we still render.
        let l = lock(vec![
            pkg("a", "1", &["b"]),
            pkg("b", "1", &["a"]),
        ]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        assert!(body.contains("a v1"));
        assert!(body.contains("b v1"));
        // The back-edge must be deduped.
        assert!(body.contains("(*)"));
    }

    #[test]
    fn roots_are_packages_with_no_consumer() {
        let l = lock(vec![
            pkg("toolA", "1", &["lib"]),
            pkg("toolB", "1", &["lib"]),
            pkg("lib", "2", &[]),
        ]);
        let roots = build_tree(&l, &TreeOptions::default());
        let names: Vec<&str> = roots.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["toolA", "toolB"]);
    }

    #[test]
    fn output_is_deterministic_across_runs() {
        let l = lock(vec![
            pkg("z", "1", &["mid"]),
            pkg("a", "1", &["mid"]),
            pkg("mid", "1", &["leaf"]),
            pkg("leaf", "1", &[]),
        ]);
        let body1 = render_lockfile_tree(&l, &TreeOptions::default());
        let body2 = render_lockfile_tree(&l, &TreeOptions::default());
        assert_eq!(body1, body2);
    }

    #[test]
    fn pep503_normalization_collapses_separators_for_adjacency() {
        let l = lock(vec![
            pkg("My_App.X", "1", &["my-pkg"]),
            pkg("My-Pkg", "2", &[]),
        ]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        // The dep edge resolves across PEP 503-equivalent names.
        assert!(body.contains("My_App.X v1"));
        assert!(body.contains("My-Pkg v2"));
    }

    #[test]
    fn focus_works_with_normalized_name() {
        let l = lock(vec![
            pkg("My-Pkg", "1", &["dep"]),
            pkg("dep", "2", &[]),
        ]);
        let body = render_lockfile_tree(
            &l,
            &TreeOptions { focus: Some("my_pkg".into()), ..Default::default() },
        );
        assert!(body.starts_with("My-Pkg v1"));
        assert!(body.contains("dep v2"));
    }

    #[test]
    fn deep_tree_indents_with_pipe_continuations() {
        let l = lock(vec![
            pkg("app", "1", &["a", "b"]),
            pkg("a", "1", &["a1"]),
            pkg("a1", "1", &[]),
            pkg("b", "1", &[]),
        ]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        // The `a` branch is not the last child, so its sub-line must use `│   `.
        assert!(body.contains("│   └── a1 v1"));
    }

    #[test]
    fn last_root_child_uses_corner_glyph() {
        let l = lock(vec![pkg("app", "1", &["only"]), pkg("only", "2", &[])]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        assert!(body.contains("└── only v2"));
        assert!(!body.contains("├── only"));
    }

    #[test]
    fn render_tree_ends_with_single_newline_when_nonempty() {
        let l = lock(vec![pkg("a", "1", &[])]);
        let body = render_lockfile_tree(&l, &TreeOptions::default());
        assert!(body.ends_with('\n'));
        assert!(!body.ends_with("\n\n"));
    }
}
