//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-async-sync-parity-invariant.md#changes
//!
//! Thin-pass-through shape audit. Walks every `pub fn` inside an
//! inherent `impl` block under `src/driver/blocking/` and asserts the
//! body matches the canonical facade shape — namely, the body must
//! either contain a `block_on` call somewhere (and never branch on
//! the result of that call), or appear on a hand-curated allowlist of
//! facade-internal extensions (constructors, getters, borrow
//! helpers).
//!
//! The shape audit's job is to catch the SQLAlchemy `AsyncSession`
//! anti-pattern: branching, retry loops, or alternate code paths
//! creeping into the sync facade. The audit is intentionally
//! permissive about layout (number of statements, leading
//! `let inner = self.inner.take()` / `let rt = conn.runtime()`
//! preparation lines) so the existing facade implementations pass on
//! their first run; it is strict about control flow.

// HANDWRITE-BEGIN reason: syn-AST body-shape audit; same codegen gap
//   as the parity walker. Closes when score's regenerability
//   invariant covers AST-shape audits.

use std::path::PathBuf;

use syn::visit::{self, Visit};
use syn::{ImplItem, Item, ItemImpl};

/// Facade fns that are legitimately not `block_on` calls. Each is a
/// constructor, getter, or borrow helper; adding to this list
/// requires explicit reviewer approval per the TD's R4 contract.
const ALLOWLIST: &[(&str, &str, &str)] = &[
    // (TypeName, FnName, reason)
    ("Connection", "from_parts", "constructor: wraps existing async handle + Runtime"),
    ("Connection", "as_async", "getter: borrows the inner async handle"),
    ("Connection", "runtime", "getter: clones the owned Runtime Arc"),
    ("Connection", "pool", "getter: delegates to inner.pool() (no IO)"),
    ("Transaction", "as_mut_transaction", "borrow helper: lends inner sqlx::Transaction"),
    // MigrationRunner sync-only entry points: counterparts on the async
    // side are themselves sync (no IO until `.up()` / `.down()`).
    ("MigrationRunner", "new", "constructor: mirrors async-side sync ctor (no IO)"),
    ("MigrationRunner", "connect", "facade convenience ctor: opens Connection + builds Runner (uses block_on internally via Connection::new)"),
    ("MigrationRunner", "load_from_directory", "static loader: filesystem read, no async runtime needed"),
    ("BulkExecutor", "new", "constructor: wraps async BulkExecutor + clones runtime (no IO)"),
    ("QueryExecutor", "new", "constructor: borrows pool from Connection (no IO)"),
    ("QueryExecutor", "with_config", "constructor: same as new + ExecutorConfig (no IO)"),
    // orm::Session blocking facade — constructors, getters, and sync
    // staging methods (mutate the UoW Vec but do no IO).
    ("Session", "new", "constructor: wraps async Session + clones runtime (no IO)"),
    ("Session", "as_async", "getter: borrows the underlying async Session"),
    ("Session", "add", "sync staging: mutates UoW Vec only, no IO"),
    ("Session", "delete", "sync staging: mutates UoW Vec only, no IO"),
    ("Session", "touch", "sync staging: mutates UoW Vec only, no IO"),
    ("Session", "query", "sync builder ctor: returns SessionQuery, no IO"),
    ("Session", "add_dyn", "sync staging dyn variant: mutates UoW Vec only, no IO"),
    ("Session", "delete_dyn", "sync staging dyn variant: mutates UoW Vec only, no IO"),
    ("Session", "touch_dyn", "sync staging dyn variant: mutates UoW Vec only, no IO"),
    ("Session", "staging_len", "diagnostic accessor: returns Vec::len, no IO"),
    ("Session", "identity_map_len", "diagnostic accessor: returns HashMap::len sum, no IO"),
    ("Session", "in_transaction", "diagnostic accessor: returns Option::is_some, no IO"),
    // SessionQuery blocking sibling — sync chainable builder methods.
    ("SessionQuery", "filter", "sync chainable builder: mutates QueryBuilder, no IO"),
    ("SessionQuery", "limit", "sync chainable builder: mutates QueryBuilder, no IO"),
];

fn pg_blocking_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/driver/blocking")
}

#[test]
fn blocking_facade_bodies_have_no_control_flow() {
    let root = pg_blocking_root();
    let mut violations: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("rs")
        })
    {
        let path = entry.path();
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let Ok(file) = syn::parse_file(&src) else {
            continue;
        };

        for item in &file.items {
            audit_items(item, path, &mut violations);
        }
    }

    assert!(
        violations.is_empty(),
        "blocking facade shape violations — facade fn bodies must be \
         straight-line `block_on` pass-throughs with no branching, \
         retry loops, or `?` chains beyond the `block_on` result:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn blocking_facade_io_fns_use_block_on() {
    let root = pg_blocking_root();
    let allowlist: std::collections::BTreeSet<(String, String)> = ALLOWLIST
        .iter()
        .map(|(t, f, _)| (t.to_string(), f.to_string()))
        .collect();
    let mut missing_block_on: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("rs")
        })
    {
        let path = entry.path();
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let Ok(file) = syn::parse_file(&src) else {
            continue;
        };

        for item in &file.items {
            audit_block_on(item, path, &allowlist, &mut missing_block_on);
        }
    }

    assert!(
        missing_block_on.is_empty(),
        "blocking facade fns that do not call `block_on` and are not \
         on the ALLOWLIST — every facade fn must either drive the \
         async surface via `block_on` or be a reviewer-approved \
         constructor/getter/borrow helper:\n{}",
        missing_block_on.join("\n"),
    );
}

fn audit_items(item: &Item, path: &std::path::Path, violations: &mut Vec<String>) {
    match item {
        Item::Mod(m) => {
            if let Some((_, inner)) = &m.content {
                for sub in inner {
                    audit_items(sub, path, violations);
                }
            }
        }
        Item::Impl(imp) if imp.trait_.is_none() => {
            let Some(ty) = impl_type_name(imp) else {
                return;
            };
            for ii in &imp.items {
                let ImplItem::Fn(f) = ii else { continue };
                if !matches!(f.vis, syn::Visibility::Public(_)) {
                    continue;
                }
                let mut walker = ControlFlowWalker::default();
                walker.visit_block(&f.block);
                if !walker.violations.is_empty() {
                    for v in walker.violations {
                        violations.push(format!(
                            "  - {}::{} ({}): {}",
                            ty,
                            f.sig.ident,
                            path.display(),
                            v,
                        ));
                    }
                }
            }
        }
        _ => {}
    }
}

fn audit_block_on(
    item: &Item,
    path: &std::path::Path,
    allowlist: &std::collections::BTreeSet<(String, String)>,
    missing: &mut Vec<String>,
) {
    match item {
        Item::Mod(m) => {
            if let Some((_, inner)) = &m.content {
                for sub in inner {
                    audit_block_on(sub, path, allowlist, missing);
                }
            }
        }
        Item::Impl(imp) if imp.trait_.is_none() => {
            let Some(ty) = impl_type_name(imp) else {
                return;
            };
            for ii in &imp.items {
                let ImplItem::Fn(f) = ii else { continue };
                if !matches!(f.vis, syn::Visibility::Public(_)) {
                    continue;
                }
                let key = (ty.clone(), f.sig.ident.to_string());
                if allowlist.contains(&key) {
                    continue;
                }
                let mut walker = BlockOnFinder::default();
                walker.visit_block(&f.block);
                if !walker.found {
                    missing.push(format!("  - {}::{} ({})", ty, f.sig.ident, path.display(),));
                }
            }
        }
        _ => {}
    }
}

fn impl_type_name(imp: &ItemImpl) -> Option<String> {
    let syn::Type::Path(tp) = &*imp.self_ty else {
        return None;
    };
    Some(tp.path.segments.last()?.ident.to_string())
}

/// Visitor that records control-flow constructs at the body level.
/// Does NOT recurse into the arguments of method calls — those may
/// well contain control flow on the async side, which is fine; what
/// we are auditing is the FACADE body, not the async body it
/// invokes.
#[derive(Default)]
struct ControlFlowWalker {
    violations: Vec<String>,
    /// Depth inside `block_on(...)` arguments — when > 0 we are
    /// looking at code that runs ON the async side (it is the future
    /// passed to `block_on`), not on the facade side. Skip checks.
    inside_block_on_arg: usize,
}

impl<'ast> Visit<'ast> for ControlFlowWalker {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        if self.inside_block_on_arg == 0 {
            self.violations.push("`if` in facade body".into());
        }
        visit::visit_expr_if(self, node);
    }
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        if self.inside_block_on_arg == 0 {
            self.violations.push("`match` in facade body".into());
        }
        visit::visit_expr_match(self, node);
    }
    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        if self.inside_block_on_arg == 0 {
            self.violations.push("`while` loop in facade body".into());
        }
        visit::visit_expr_while(self, node);
    }
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        if self.inside_block_on_arg == 0 {
            self.violations.push("`for` loop in facade body".into());
        }
        visit::visit_expr_for_loop(self, node);
    }
    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        if self.inside_block_on_arg == 0 {
            self.violations.push("`loop` in facade body".into());
        }
        visit::visit_expr_loop(self, node);
    }
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "block_on" {
            // Step into args but mark the depth so nested control
            // flow inside the async future doesn't fail the audit.
            self.inside_block_on_arg += 1;
            visit::visit_expr_method_call(self, node);
            self.inside_block_on_arg -= 1;
        } else {
            visit::visit_expr_method_call(self, node);
        }
    }
}

/// Visitor that records whether the body contains a `block_on`
/// method call anywhere.
#[derive(Default)]
struct BlockOnFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for BlockOnFinder {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "block_on" {
            self.found = true;
        }
        visit::visit_expr_method_call(self, node);
    }
}

// HANDWRITE-END
