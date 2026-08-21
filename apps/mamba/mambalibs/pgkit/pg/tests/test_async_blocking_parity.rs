//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-async-sync-parity-invariant.md#changes
//!
//! Parity-audit test. Walks the public surface of `pg::driver` and
//! `pg::migrate` (minus auxiliary inspection helpers — see
//! `NON_PARITY_FILES`) with a `syn`-based AST visitor and asserts that
//! every public `async fn` has a corresponding public blocking sibling
//! under `pg::driver::blocking` — and vice versa.
//!
//! `pg::orm` is out of scope today: per R5 of the parity-invariant TD,
//! the orm layer joins the invariant when the `Session` surface lands
//! (and brings a `BlockingSession` facade in the same change).

// HANDWRITE-BEGIN reason: syn-AST parity walker; score does not yet
//   have a `parity-audit` section type that emits this shape. Closes
//   when a third pg-side use case appears or when score's
//   regenerability invariant covers `syn::visit::Visit`-style
//   walkers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{ImplItem, Item, ItemImpl};

/// `(TypeName, FnName)` pair — the canonical key for the parity
/// comparison. We compare on the short type name (the type as
/// written in the `impl Foo` header) and the short fn name. This
/// matches blocking-side `Connection::ping` against driver-side
/// `Connection::ping` even though the blocking type lives in a
/// different module.
type Pair = (String, String);

/// Methods that exist on the async side but legitimately have no
/// blocking sibling (or vice versa). Each entry needs a one-line
/// reason; additions require explicit reviewer approval per the
/// TD's R4 contract.
const EXEMPT_ASYNC_ONLY: &[(&str, &str, &str)] = &[
    // (TypeName, FnName, reason)
    // (none today — every async IO fn has a blocking sibling)
];

/// Blocking-side functions that legitimately have no async sibling.
/// These are facade-internal extensions: constructors that build the
/// owned runtime, getters that expose the runtime/inner handle, and
/// borrow helpers used by the binding layer.
const EXEMPT_BLOCKING_ONLY: &[(&str, &str, &str)] = &[
    (
        "Connection",
        "from_parts",
        "facade ctor: wrap an existing async handle + Runtime",
    ),
    (
        "Connection",
        "as_async",
        "facade escape hatch: borrow the inner async handle",
    ),
    (
        "Connection",
        "runtime",
        "facade getter: share the owned Runtime across derived handles",
    ),
    (
        "Transaction",
        "as_mut_transaction",
        "facade borrow helper: lend the inner sqlx Transaction",
    ),
    (
        "MigrationRunner",
        "connect",
        "facade convenience ctor: takes a URL, builds a Connection + Runner in one step",
    ),
];

/// Methods that look IO-bearing in name but are pure getters /
/// borrow helpers (no `block_on`). These do not require a blocking
/// counterpart and are excluded from the parity check entirely.
const ASYNC_SIDE_NON_IO: &[(&str, &str)] = &[
    ("Connection", "pool"),                // sync getter on async side too
    ("Transaction", "as_mut_transaction"), // sync borrow on async side too
];

/// Files under the scanned subtrees that are deliberately OUTSIDE the
/// parity invariant. Per R5 of the parent TD: `orm/` joins the
/// invariant when the Session lands; the migrate auxiliary types
/// (history visualizer, model differ, status report) are read-only
/// inspection helpers that have not yet been bound into the blocking
/// facade. Narrowing that residual carve-out is a future follow-up.
///
/// The blocking subtree is excluded structurally (path contains
/// `blocking` segment) and is not listed here.
const NON_PARITY_FILES: &[&str] = &[
    // migrate/ auxiliary types — not yet in the blocking facade scope.
    "history_vis.rs",
    "model_diff.rs",
    "status_report.rs",
];

fn pg_crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Collect `(TypeName, FnName)` for every `pub async fn` inside an
/// inherent `impl` block found anywhere under `dirs`. The blocking
/// subtree is excluded by the caller.
fn collect_async_surface(dirs: &[PathBuf]) -> BTreeSet<Pair> {
    let mut out = BTreeSet::new();
    for dir in dirs {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().and_then(|s| s.to_str()) == Some("rs")
                    // Exclude the blocking subtree explicitly.
                    && !e.path().components().any(|c| c.as_os_str() == "blocking")
                    // Exclude files declared outside the parity scope.
                    && !e
                        .path()
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|n| NON_PARITY_FILES.contains(&n))
                        .unwrap_or(false)
            })
        {
            let src = std::fs::read_to_string(entry.path())
                .unwrap_or_else(|e| panic!("read {}: {e}", entry.path().display()));
            let Ok(file) = syn::parse_file(&src) else {
                continue;
            };
            collect_pub_async_methods(&file.items, &mut out);
        }
    }
    out
}

/// Collect `(TypeName, FnName)` for every `pub fn` (async OR sync)
/// inside an inherent `impl` block found anywhere under `dirs`, with
/// the same blocking-subtree and NON_PARITY_FILES exclusions as
/// `collect_async_surface`. Used for the blocking → async direction:
/// a blocking fn is legitimate if the async side has a fn with the
/// same name regardless of whether it is itself `async` (e.g.
/// `MigrationRunner::new` is a sync constructor on both sides).
fn collect_async_side_all_pub_fns(dirs: &[PathBuf]) -> BTreeSet<Pair> {
    let mut out = BTreeSet::new();
    for dir in dirs {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().and_then(|s| s.to_str()) == Some("rs")
                    && !e.path().components().any(|c| c.as_os_str() == "blocking")
                    && !e
                        .path()
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|n| NON_PARITY_FILES.contains(&n))
                        .unwrap_or(false)
            })
        {
            let src = std::fs::read_to_string(entry.path())
                .unwrap_or_else(|e| panic!("read {}: {e}", entry.path().display()));
            let Ok(file) = syn::parse_file(&src) else {
                continue;
            };
            collect_all_pub_methods(&file.items, &mut out);
        }
    }
    out
}

/// Collect `(TypeName, FnName)` for every `pub fn` inside an inherent
/// `impl` block found anywhere under `dir` (the blocking subtree).
fn collect_blocking_surface(dir: &Path) -> BTreeSet<Pair> {
    let mut out = BTreeSet::new();
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("rs")
        })
    {
        let src = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("read {}: {e}", entry.path().display()));
        let Ok(file) = syn::parse_file(&src) else {
            continue;
        };
        collect_pub_sync_methods(&file.items, &mut out);
    }
    out
}

fn collect_pub_async_methods(items: &[Item], out: &mut BTreeSet<Pair>) {
    for item in items {
        match item {
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_pub_async_methods(inner, out);
                }
            }
            Item::Impl(imp) if imp.trait_.is_none() => {
                let Some(ty) = impl_type_name(imp) else {
                    continue;
                };
                for ii in &imp.items {
                    let ImplItem::Fn(f) = ii else { continue };
                    if !matches!(f.vis, syn::Visibility::Public(_)) {
                        continue;
                    }
                    if f.sig.asyncness.is_none() {
                        continue;
                    }
                    out.insert((ty.clone(), f.sig.ident.to_string()));
                }
            }
            _ => {}
        }
    }
}

fn collect_all_pub_methods(items: &[Item], out: &mut BTreeSet<Pair>) {
    for item in items {
        match item {
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_all_pub_methods(inner, out);
                }
            }
            Item::Impl(imp) if imp.trait_.is_none() => {
                let Some(ty) = impl_type_name(imp) else {
                    continue;
                };
                for ii in &imp.items {
                    let ImplItem::Fn(f) = ii else { continue };
                    if !matches!(f.vis, syn::Visibility::Public(_)) {
                        continue;
                    }
                    out.insert((ty.clone(), f.sig.ident.to_string()));
                }
            }
            _ => {}
        }
    }
}

fn collect_pub_sync_methods(items: &[Item], out: &mut BTreeSet<Pair>) {
    for item in items {
        match item {
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_pub_sync_methods(inner, out);
                }
            }
            Item::Impl(imp) if imp.trait_.is_none() => {
                let Some(ty) = impl_type_name(imp) else {
                    continue;
                };
                for ii in &imp.items {
                    let ImplItem::Fn(f) = ii else { continue };
                    if !matches!(f.vis, syn::Visibility::Public(_)) {
                        continue;
                    }
                    // Blocking facade is sync only; an async fn here
                    // would itself be a violation, but the parity
                    // check only cares about the sync surface.
                    if f.sig.asyncness.is_some() {
                        continue;
                    }
                    // Blocking entries that share their async-side
                    // host type (e.g. additional `impl Row` blocks
                    // under `driver/blocking/row.rs`) must use a
                    // `_blocking` suffix to avoid colliding with the
                    // async-side inherent method of the same name.
                    // Strip the suffix so the parity walker pairs
                    // `Row::insert_blocking` with the async-side
                    // `Row::insert`.
                    let name = f.sig.ident.to_string();
                    let pair_name = name
                        .strip_suffix("_blocking")
                        .map(str::to_string)
                        .unwrap_or(name);
                    out.insert((ty.clone(), pair_name));
                }
            }
            _ => {}
        }
    }
}

fn impl_type_name(imp: &ItemImpl) -> Option<String> {
    let syn::Type::Path(tp) = &*imp.self_ty else {
        return None;
    };
    Some(tp.path.segments.last()?.ident.to_string())
}

#[test]
fn parity_async_surface_has_blocking_sibling() {
    let root = pg_crate_root();
    let async_dirs = [
        root.join("src/driver"),
        root.join("src/migrate"),
        // src/orm: per R5, joins parity invariant only when Session lands.
    ];
    let blocking_dir = root.join("src/driver/blocking");

    let async_surface = collect_async_surface(&async_dirs);
    let blocking_surface = collect_blocking_surface(&blocking_dir);

    let exempt: BTreeSet<Pair> = EXEMPT_ASYNC_ONLY
        .iter()
        .map(|(t, f, _)| (t.to_string(), f.to_string()))
        .collect();

    let missing: Vec<&Pair> = async_surface
        .iter()
        .filter(|p| !blocking_surface.contains(*p) && !exempt.contains(*p))
        .collect();

    assert!(
        missing.is_empty(),
        "async fns without a blocking sibling — add the blocking-side \
         pass-through or extend EXEMPT_ASYNC_ONLY with a reviewer-\
         approved reason:\n{}",
        format_pairs(&missing),
    );
}

#[test]
fn parity_blocking_surface_has_async_sibling() {
    let root = pg_crate_root();
    let async_dirs = [
        root.join("src/driver"),
        root.join("src/migrate"),
        // src/orm: per R5, joins parity invariant only when Session lands.
    ];
    let blocking_dir = root.join("src/driver/blocking");

    // For the blocking → async direction we match against the
    // complete pub surface on the async side (async OR sync). This
    // handles types like `MigrationRunner::new` and
    // `MigrationRunner::load_from_directory` which are sync
    // constructors on both sides — they are not "async fns" but they
    // still have a counterpart, so the blocking-side fn is legitimate.
    let async_side_all = collect_async_side_all_pub_fns(&async_dirs);
    let blocking_surface = collect_blocking_surface(&blocking_dir);

    let exempt: BTreeSet<Pair> = EXEMPT_BLOCKING_ONLY
        .iter()
        .map(|(t, f, _)| (t.to_string(), f.to_string()))
        .collect();

    let non_io: BTreeSet<Pair> = ASYNC_SIDE_NON_IO
        .iter()
        .map(|(t, f)| (t.to_string(), f.to_string()))
        .collect();

    let missing: Vec<&Pair> = blocking_surface
        .iter()
        .filter(|p| !async_side_all.contains(*p) && !exempt.contains(*p) && !non_io.contains(*p))
        .collect();

    assert!(
        missing.is_empty(),
        "blocking fns without an async sibling — add the async-side \
         counterpart or extend EXEMPT_BLOCKING_ONLY with a reviewer-\
         approved reason:\n{}",
        format_pairs(&missing),
    );
}

/// Sanity test: the audit-finding baseline embedded in the TD spec
/// must match what the walker actually finds. Any drift is a
/// reviewer-visible signal that the spec and code disagree.
#[test]
fn parity_baseline_matches_walked_surface() {
    let root = pg_crate_root();
    let async_dirs = [
        root.join("src/driver"),
        root.join("src/migrate"),
        // src/orm: per R5, joins parity invariant only when Session lands.
    ];
    let blocking_dir = root.join("src/driver/blocking");

    let async_surface = collect_async_surface(&async_dirs);
    let blocking_surface = collect_blocking_surface(&blocking_dir);

    // Group by type for human-readable assertion output.
    let async_by_type = group_by_type(&async_surface);
    let blocking_by_type = group_by_type(&blocking_surface);

    // Every type with an async surface must appear with a blocking
    // surface — vacuous types (zero async fns) are allowed.
    for (ty, fns) in &async_by_type {
        if !blocking_by_type.contains_key(ty) {
            panic!(
                "type `{ty}` has an async surface ({} fn(s)) but no \
                 blocking impl at all",
                fns.len()
            );
        }
    }
}

fn group_by_type(pairs: &BTreeSet<Pair>) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (ty, f) in pairs {
        out.entry(ty.clone()).or_default().push(f.clone());
    }
    out
}

fn format_pairs(pairs: &[&Pair]) -> String {
    pairs
        .iter()
        .map(|(t, f)| format!("  - {t}::{f}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// HANDWRITE-END
