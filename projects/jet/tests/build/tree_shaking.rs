// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Tree-shaking validation suite — fixture-based snapshot tests for
//! `crate::bundler::tree_shake`. This is the **R1 + R2 foundation
//! slice** of #1249:
//!
//! - **R1** — fixture-based test suite under `projects/jet/tests/`.
//! - **R2** — snapshot-based assertions on the analyzer + shaker output.
//! - **R3** (next slice) — bundle-size baseline regression gate per
//!   fixture.
//! - **R4** (final slice) — wire the mini-react example as a realistic
//!   end-to-end fixture.
//!
//! Each test pins one of the patterns the issue lists in scope. The
//! shape under snapshot is intentionally narrow:
//!
//! ```json
//! {
//!   "used_exports":      { "<module-path>": ["name", ...] },
//!   "eliminated_modules": ["<module-path>"],
//!   "eliminated_bytes":   123
//! }
//! ```
//!
//! Tests use **synthetic in-memory module paths** (no tempdir, no
//! disk) so they stay hermetic and fast. `read_package_side_effects`
//! is exercised separately in the bundler's own unit tests; the
//! suite here drives the pure analytical surface
//! (`analyze_used_exports` / `shake_module` / `module_has_side_effects`)
//! that the bundler chains together.
//!
//! Snapshots are written to
//! `projects/jet/tests/__snapshots__/tree_shaking__<name>.json`. First
//! run writes the snapshot; subsequent runs diff against it. Set
//! `JET_SNAPSHOT_UPDATE=1` to overwrite when changes are intentional.

use jet::bundler::tree_shake::{
    analyze_used_exports, analyze_used_exports_from, has_side_effects, module_has_side_effects,
    shake_module, SideEffectsDecl, TreeShakeResult,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ─── Snapshot harness (self-contained) ──────────────────────────────────────
//
// Mirrors `tests/common/snapshot.rs` but inlined so this test binary
// doesn't pull the wider `tests/common/mod.rs` browser + wasm_build
// surface into its compile graph.

fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("__snapshots__")
}

fn snapshot_eq(name: &str, actual: &Value) {
    let dir = snapshot_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("creating {}: {e}", dir.display()));
    }
    let path = dir.join(format!("{name}.json"));
    let serialized = canonicalize(actual);
    let update = std::env::var("JET_SNAPSHOT_UPDATE").is_ok();

    match (path.exists(), update) {
        (false, _) | (true, true) => {
            std::fs::write(&path, &serialized)
                .unwrap_or_else(|e| panic!("writing {}: {e}", path.display()));
            eprintln!(
                "[snapshot] wrote {} ({} bytes)",
                path.display(),
                serialized.len()
            );
        }
        (true, false) => {
            let expected = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            if expected != serialized {
                panic!(
                    "snapshot drift at {}\n\
                     → JET_SNAPSHOT_UPDATE=1 cargo test ... to accept the new shape\n\
                     \nexpected ({} bytes):\n{}\n\nactual ({} bytes):\n{}\n",
                    path.display(),
                    expected.len(),
                    expected,
                    serialized.len(),
                    serialized,
                );
            }
        }
    }
}

fn canonicalize(v: &Value) -> String {
    let sorted = sort_keys(v);
    let mut out = serde_json::to_string_pretty(&sorted)
        .expect("pretty-printing a serde_json::Value can't fail");
    out.push('\n');
    out
}

fn sort_keys(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut pairs: Vec<(String, Value)> =
                map.iter().map(|(k, v)| (k.clone(), sort_keys(v))).collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = serde_json::Map::new();
            for (k, v) in pairs {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_keys).collect()),
        other => other.clone(),
    }
}

// ─── TreeShakeResult → snapshot Value ───────────────────────────────────────
//
// Convert the result into a deterministic JSON shape:
// - `used_exports` keys are module paths (string), values are sorted
//   arrays of export names.
// - `eliminated_modules` is a sorted array of module paths.
// - `eliminated_bytes` is the raw u64.

fn result_to_value(result: &TreeShakeResult) -> Value {
    let mut used: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, names) in &result.used_exports {
        let mut sorted: Vec<String> = names.iter().cloned().collect();
        sorted.sort();
        used.insert(path.to_string_lossy().into_owned(), sorted);
    }

    let mut eliminated: Vec<String> = result
        .eliminated_modules
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    eliminated.sort();

    json!({
        "used_exports": used,
        "eliminated_modules": eliminated,
        "eliminated_bytes": result.eliminated_bytes,
    })
}

fn fixture(modules: &[(&str, &str)]) -> Vec<(PathBuf, String)> {
    modules
        .iter()
        .map(|(p, src)| (PathBuf::from(p), (*src).to_string()))
        .collect()
}

// ─── Patterns ───────────────────────────────────────────────────────────────

/// **Pattern 1 — Unused named exports must be stripped.**
///
/// Entry imports only `used` from a helper that exports both `used`
/// and `unused`. The shaker should drop the `unused` export from the
/// helper's source and `analyze_used_exports` should record only
/// `used` for the helper module.
#[test]
fn unused_named_exports_are_stripped() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { used } from './helper';\nconsole.log(used());\n",
        ),
        (
            "/fixture/helper.js",
            "export const used = () => 1;\nexport const unused = () => 2;\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");
    snapshot_eq(
        "tree_shaking__unused_named_exports__analysis",
        &result_to_value(&result),
    );

    // Pin the post-shake source for the helper too — that's the half
    // of the contract the bundler actually emits.
    let helper_path = PathBuf::from("/fixture/helper.js");
    let helper_src = &modules[1].1;
    let used = result
        .used_exports
        .get(&helper_path)
        .cloned()
        .unwrap_or_default();
    let shaken = shake_module(helper_src, &helper_path, &used);
    snapshot_eq(
        "tree_shaking__unused_named_exports__shaken_helper",
        &Value::String(shaken),
    );
}

/// **Pattern 2 — `sideEffects: false` honored.**
///
/// A module with no used imports AND no top-level side effects must
/// be eliminated. We check both `module_has_side_effects` (the
/// per-module gate the bundler consults) and `analyze_used_exports`
/// (the graph-level analyzer that promotes such modules into
/// `eliminated_modules`).
#[test]
fn side_effect_free_module_is_eliminated() {
    let modules = fixture(&[
        ("/fixture/entry.js", "console.log('hello');\n"),
        (
            "/fixture/orphan.js",
            "export const a = 1;\nexport const b = 2;\n",
        ),
    ]);

    // Per-module gate: orphan.js, declared sideEffects: false at the
    // package level, has no top-level side effects either, so it's
    // safely dropable.
    let orphan = Path::new("/fixture/orphan.js");
    let has_se = module_has_side_effects(&modules[1].1, orphan, &SideEffectsDecl::None);
    assert!(
        !has_se,
        "module_has_side_effects must return false when sideEffects:false and code is pure",
    );

    // Graph-level: orphan has exports but nobody imports them, so it
    // lands in `eliminated_modules`.
    let result = analyze_used_exports(&modules).expect("analyze_used_exports");
    snapshot_eq(
        "tree_shaking__side_effect_free_module__analysis",
        &result_to_value(&result),
    );
}

/// **Pattern 3 — Side-effect-full module preserved.**
///
/// Even if no exports are used, a top-level statement (`console.log`,
/// IIFE, polyfill init) must keep the module in the bundle. This is
/// the conservative branch — `has_side_effects` returns `true`, and
/// the analyzer must NOT add the module to `eliminated_modules`.
#[test]
fn side_effect_full_module_is_preserved() {
    let modules = fixture(&[
        ("/fixture/entry.js", "console.log('entry');\n"),
        (
            "/fixture/polyfill.js",
            // Top-level statement — classic polyfill shape.
            "globalThis.__polyfilled = true;\nexport const ignored = 1;\n",
        ),
    ]);

    assert!(
        has_side_effects(&modules[1].1),
        "polyfill.js has a top-level assignment — has_side_effects must return true",
    );

    // Even when the package declares `sideEffects: false`, a TUI/web
    // bundler that bypasses package metadata sees no code-level side
    // effects only if `has_side_effects` says so. Pin both signals.
    let polyfill = Path::new("/fixture/polyfill.js");
    let has_se_with_decl_false =
        module_has_side_effects(&modules[1].1, polyfill, &SideEffectsDecl::None);
    assert!(
        !has_se_with_decl_false,
        "sideEffects:false overrides code analysis — this is documented bundler behavior",
    );
    let has_se_with_decl_all =
        module_has_side_effects(&modules[1].1, polyfill, &SideEffectsDecl::All);
    assert!(
        has_se_with_decl_all,
        "sideEffects:true (default) preserves the module on code-level side effects",
    );

    // Snapshot the graph-level result. With no decl input, the
    // analyzer relies on `has_side_effects` and must keep
    // polyfill.js out of `eliminated_modules`.
    let result = analyze_used_exports(&modules).expect("analyze_used_exports");
    assert!(
        !result
            .eliminated_modules
            .iter()
            .any(|p| p == Path::new("/fixture/polyfill.js")),
        "polyfill.js must not be eliminated — it has top-level side effects",
    );
    snapshot_eq(
        "tree_shaking__side_effect_full_module__analysis",
        &result_to_value(&result),
    );
}

/// **Pattern 4 — Re-export chains: `export { x } from './y'` with
/// partial usage.**
///
/// Setup:
///
/// - `entry.js` imports only `a` from `./barrel`.
/// - `barrel.js` re-exports both `a` from `./a` and `b` from `./b`.
/// - `a.js` and `b.js` each export one constant.
///
/// Behavior after #1342: the analyzer threads named re-exports, so
/// `a.js` keeps its `a` export (consumed transitively through the
/// barrel) while `b.js` — whose `b` is re-exported by the barrel but
/// never consumed by `entry.js` — remains eligible for elimination.
/// The `barrel.js` `export { a } from './a'` line stays alive and
/// the snapshot records the shrunken `eliminated_modules` set.
#[test]
fn reexport_chain_partial_usage_baseline() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { a } from './barrel';\nconsole.log(a);\n",
        ),
        (
            "/fixture/barrel.js",
            "export { a } from './a';\nexport { b } from './b';\n",
        ),
        ("/fixture/a.js", "export const a = 1;\n"),
        ("/fixture/b.js", "export const b = 2;\n"),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    // Sanity: the barrel surface is reached — entry's `import { a }`
    // marks barrel's `a` export as used. This is the part the
    // analyzer DOES handle correctly today.
    let barrel_used = result
        .used_exports
        .get(&PathBuf::from("/fixture/barrel.js"))
        .cloned()
        .unwrap_or_default();
    assert!(
        barrel_used.contains("a"),
        "entry's `import {{ a }} from './barrel'` must mark barrel's `a` as used; got {barrel_used:?}",
    );

    // Per #1342: re-exports are now threaded — `a.js`'s `a` lands
    // in `used_exports` and the module is no longer eliminated. The
    // unreferenced `b.js` is still side-effect-free with no used
    // exports, so it stays eligible for elimination.
    let a = PathBuf::from("/fixture/a.js");
    let b = PathBuf::from("/fixture/b.js");
    let a_used = result.used_exports.get(&a).cloned().unwrap_or_default();
    assert!(
        a_used.contains("a"),
        "a.js's `a` export must be threaded through the barrel re-export; got {a_used:?}",
    );
    assert!(
        !result.eliminated_modules.contains(&a),
        "a.js must not be eliminated — its `a` export is consumed via the barrel; got {:?}",
        result.eliminated_modules,
    );
    assert!(
        result.eliminated_modules.contains(&b),
        "b.js must still be eliminated — barrel re-exports `b` but nobody consumes it; got {:?}",
        result.eliminated_modules,
    );

    snapshot_eq(
        "tree_shaking__reexport_chain_partial_usage__analysis",
        &result_to_value(&result),
    );
}

/// **Pattern 4b — `export * from './y'` (star re-export, used-name
/// propagation).**
///
/// `export * from '...'` is the namespace-passthrough form of a
/// barrel: every named export of the leaf becomes a named export of
/// the barrel, except for `default` (which the spec excludes from
/// `export *`). Demand-driven analysis can still thread the specific
/// name selected by a downstream consumer, so a named import of `a`
/// keeps `a` alive without retaining unrelated sibling exports.
///
/// Setup:
///
/// - `entry.js` consumes one symbol (`a`) from `./barrel`.
/// - `barrel.js` star-re-exports every name of `./leaf`.
/// - `leaf.js` exports `a`, `b`, and a default — only `a` and `b`
///   are visible to the star, the default is excluded.
///
/// Expected:
///
/// - `leaf.js`'s `a` is in `used_exports` because it flows through
///   the star barrel to the entry import.
/// - `leaf.js`'s `b` is NOT in `used_exports` because nothing consumes
///   that exported name.
/// - `leaf.js`'s `default` is NOT in `used_exports` because
///   `export *` excludes default; consumers must use
///   `export { default } from` explicitly.
/// - `leaf.js` is NOT in `eliminated_modules`.
#[test]
fn reexport_star_marks_used_leaf_exports() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { a } from './barrel';\nconsole.log(a);\n",
        ),
        ("/fixture/barrel.js", "export * from './leaf';\n"),
        (
            "/fixture/leaf.js",
            "export const a = 1;\nexport const b = 2;\nexport default 3;\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    let leaf = PathBuf::from("/fixture/leaf.js");
    let leaf_used = result.used_exports.get(&leaf).cloned().unwrap_or_default();
    assert!(
        leaf_used.contains("a"),
        "leaf.js's `a` must be marked used via `export *`; got {leaf_used:?}",
    );
    assert!(
        !leaf_used.contains("b"),
        "leaf.js's `b` must not be marked used when only `a` is consumed through `export *`; got {leaf_used:?}",
    );
    assert!(
        !leaf_used.contains("default"),
        "leaf.js's `default` must NOT be marked used — `export *` does not re-export default; got {leaf_used:?}",
    );
    assert!(
        !result.eliminated_modules.contains(&leaf),
        "leaf.js must not be eliminated — it has used exports via the star re-export; got {:?}",
        result.eliminated_modules,
    );

    snapshot_eq(
        "tree_shaking__reexport_star_used_name__analysis",
        &result_to_value(&result),
    );
}

/// **Pattern 4c — `export { x as alias } from './y'` (renamed
/// re-export).**
///
/// When a barrel renames a leaf symbol (`export { foo as bar } from
/// './leaf'`), consumers see `bar` on the barrel; the leaf still
/// owns the symbol as `foo`. If the consumer is selective
/// (`import { bar }`), the analyzer must propagate that as `foo`
/// being used on the leaf — not `bar` (which the leaf doesn't have).
#[test]
fn reexport_renamed_threads_to_original_leaf_name() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { bar } from './barrel';\nconsole.log(bar);\n",
        ),
        (
            "/fixture/barrel.js",
            "export { foo as bar } from './leaf';\n",
        ),
        (
            "/fixture/leaf.js",
            "export const foo = 1;\nexport const other = 2;\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    let leaf = PathBuf::from("/fixture/leaf.js");
    let leaf_used = result.used_exports.get(&leaf).cloned().unwrap_or_default();
    assert!(
        leaf_used.contains("foo"),
        "leaf.js's original `foo` must be marked used (barrel exposes it as `bar`); got {leaf_used:?}",
    );
    assert!(
        !leaf_used.contains("bar"),
        "leaf.js does not own a `bar` symbol — only `foo`; got {leaf_used:?}",
    );
    assert!(
        !leaf_used.contains("other"),
        "leaf.js's unrelated `other` export must stay unused; got {leaf_used:?}",
    );
    assert!(
        !result.eliminated_modules.contains(&leaf),
        "leaf.js must not be eliminated — `foo` is consumed via the renamed barrel re-export",
    );
}

/// **Pattern 5 — Dynamic `import()` retention.** (#1344 fixed.)
///
/// `import("./lazy.js")` (the call-expression form, not the static
/// `import` declaration) loads the module's whole namespace at
/// runtime. Bundlers must retain `lazy.js` and its exports even
/// though no static binding references any of them — a code-split
/// chunk depends on the module being present in the graph.
///
/// Before #1344 the analyzer matched static imports with
/// `trimmed.starts_with("import ")` (note the trailing space).
/// Dynamic imports `import("./lazy.js")` have no space — they
/// failed that match and the module was invisible to the static
/// analyzer; combined with `lazy.js`'s side-effect-free body, it
/// was wrongly listed in `eliminated_modules`. The fix adds
/// `extract_dynamic_import_targets` which walks every `import(...)`
/// call-expression and marks the target with `"*"`. The snapshot
/// pinned here now reflects the corrected shape.
#[test]
fn dynamic_import_retained_baseline() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "const lazy = import('./lazy');\nconsole.log(lazy);\n",
        ),
        (
            "/fixture/lazy.js",
            "export const heavy = () => 'expensive';\nexport const other = 1;\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    // Contract surface: lazy.js stays in the graph and gets the
    // wildcard `"*"` so every export is retained.
    let lazy = PathBuf::from("/fixture/lazy.js");
    assert!(
        !result.eliminated_modules.contains(&lazy),
        "lazy.js must NOT be eliminated — entry.js loads it via import(); got {:?}",
        result.eliminated_modules,
    );
    let lazy_used = result.used_exports.get(&lazy).cloned().unwrap_or_default();
    assert!(
        lazy_used.contains("*"),
        "lazy.js must be marked with wildcard `*` (whole-namespace dynamic import); got {lazy_used:?}",
    );

    snapshot_eq(
        "tree_shaking__dynamic_import_retained__analysis",
        &result_to_value(&result),
    );
}

/// **Pattern 5b — `await import()` retention.** (#1344, AC.)
///
/// Same shape as the basic dynamic-import case but the call is
/// `await`-prefixed (the most common modern form). Pinned as a
/// separate fixture so a regression on either prefix shape shows
/// up loud.
#[test]
fn dynamic_import_retained_await() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "async function load() { const m = await import(\"./lazy\"); return m; }\n",
        ),
        (
            "/fixture/lazy.js",
            "export const heavy = () => 'expensive';\nexport const other = 1;\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    let lazy = PathBuf::from("/fixture/lazy.js");
    assert!(
        !result.eliminated_modules.contains(&lazy),
        "lazy.js must NOT be eliminated under `await import(...)`; got {:?}",
        result.eliminated_modules,
    );
    let lazy_used = result.used_exports.get(&lazy).cloned().unwrap_or_default();
    assert!(
        lazy_used.contains("*"),
        "lazy.js must be marked with wildcard `*` under `await import(...)`; got {lazy_used:?}",
    );

    snapshot_eq(
        "tree_shaking__dynamic_import_retained_await__analysis",
        &result_to_value(&result),
    );
}

/// **Pattern 6 — Class with unused methods (documented as not
/// supported).**
///
/// Per the issue's in-scope list: "Classes with unused methods —
/// remove if mangler supports; else document as not supported."
/// Tree-shaking lives at the export level — `export class Foo {…}`
/// is one symbol regardless of how many methods it carries. The
/// current `projects/jet/src/bundler/mangle.rs` and
/// `projects/jet/src/bundler/dce.rs` do NOT do method-level DCE, so
/// an unused method on an otherwise-used class is preserved
/// verbatim.
///
/// This test pins the "not supported" baseline:
/// 1. The whole class is retained because its export is used.
/// 2. `shake_module` does not modify the class body — both `add`
///    (used) and `sub` (unused) survive.
///
/// Future mangler work that adds method-level DCE will produce a
/// snapshot diff: the `sub` method line disappears from
/// `tree_shaking__class_unused_methods__shaken_calc`. Until then
/// the snapshot calibrates today's behavior so accidental over-
/// or under-shaking shows up loud.
#[test]
fn class_unused_methods_documented_baseline() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { Calc } from './calc';\nconst c = new Calc();\nconsole.log(c.add(1, 2));\n",
        ),
        (
            "/fixture/calc.js",
            "export class Calc {\n  add(a, b) {\n    return a + b;\n  }\n  sub(a, b) {\n    return a - b;\n  }\n}\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    // The class export `Calc` is used; calc.js stays out of
    // `eliminated_modules`.
    let calc = PathBuf::from("/fixture/calc.js");
    assert!(
        !result.eliminated_modules.contains(&calc),
        "calc.js must not be eliminated — its class export is reached by entry.js",
    );
    let calc_used = result.used_exports.get(&calc).cloned().unwrap_or_default();
    assert!(
        calc_used.contains("Calc"),
        "calc.js's `Calc` export must be marked used; got {calc_used:?}",
    );

    snapshot_eq(
        "tree_shaking__class_unused_methods__analysis",
        &result_to_value(&result),
    );

    // `shake_module` is a no-op on the class body — there's no
    // export-level pruning to do, and the function is line-oriented
    // so it can't peer into the class body even if it wanted to.
    // The unused `sub` method is preserved verbatim.
    let shaken = shake_module(&modules[1].1, &calc, &calc_used);
    assert!(
        shaken.contains("sub(a, b)"),
        "today's shake_module preserves unused methods (no method-level DCE) — pinning that baseline",
    );
    snapshot_eq(
        "tree_shaking__class_unused_methods__shaken_calc",
        &Value::String(shaken),
    );
}

/// **Pattern 7 — Mixed ESM/CJS interop.**
///
/// ESM entry imports a CJS module that assigns `exports.X` on
/// branches (an old polyfill / env-gated pattern). The analyzer
/// must:
///
/// 1. Pick up both branches' exports via `extract_cjs_export_names`
///    (which is line-oriented — both `exports.prod = …` and
///    `exports.dev = …` are detected regardless of the surrounding
///    `if` block).
/// 2. Mark only the imported name (`prod` here) as used so the
///    unused branch's export does not leak into "must keep" set.
///
/// Today's `shake_module` only strips ESM `export ` lines; CJS
/// `exports.X = …` lines survive even when the export is unused.
/// That's a known limitation — the snapshot pins it. Future work
/// to extend the shaker to CJS would shrink the
/// `mixed_esm_cjs__shaken_lib` snapshot.
#[test]
fn mixed_esm_cjs_interop_baseline() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { prod } from './lib';\nconsole.log(prod());\n",
        ),
        (
            "/fixture/lib.js",
            // Two branches assign different exports. Both are
            // statically visible to the line-oriented analyzer; only
            // `prod` is reached by entry.js's import binding.
            "if (process.env.NODE_ENV === 'production') {\n  exports.prod = () => 'fast';\n} else {\n  exports.dev = () => 'slow';\n}\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    let lib = PathBuf::from("/fixture/lib.js");
    let lib_used = result.used_exports.get(&lib).cloned().unwrap_or_default();
    assert!(
        lib_used.contains("prod"),
        "lib.js's `prod` export must be marked used (entry imports it); got {lib_used:?}",
    );
    assert!(
        !lib_used.contains("dev"),
        "lib.js's `dev` export must NOT be marked used (entry does not import it); got {lib_used:?}",
    );

    // lib.js has top-level statements (the `if` block) so
    // has_side_effects returns true and the module stays out of
    // `eliminated_modules` even though the analyzer wouldn't
    // strip the dev branch.
    assert!(
        !result.eliminated_modules.contains(&lib),
        "lib.js has top-level side effects (if-block) — must not be eliminated",
    );

    snapshot_eq(
        "tree_shaking__mixed_esm_cjs_interop__analysis",
        &result_to_value(&result),
    );

    // shake_module on a CJS module with an unused export: today's
    // shaker leaves CJS `exports.X = …` lines alone (only ESM
    // `export ` lines are stripped). Pin the verbatim survival —
    // future work to extend the shaker to CJS produces a clean
    // diff (the `dev` branch line disappears).
    let shaken = shake_module(&modules[1].1, &lib, &lib_used);
    assert!(
        shaken.contains("exports.dev = () => 'slow'"),
        "today's shake_module preserves unused CJS exports — pinning that baseline",
    );
    snapshot_eq(
        "tree_shaking__mixed_esm_cjs_interop__shaken_lib",
        &Value::String(shaken),
    );
}

/// **Pattern 8 — Conditional-exports pruning (node vs browser).**
///
/// A package's `package.json` declares
/// `{"exports": {"node": "./n.js", "browser": "./b.js"}}`. With the
/// `browser` condition active, the resolver picks `b.js`; `n.js`
/// must not leak into the bundle. The resolver side is exercised
/// in `crate::resolver` tests; the tree-shaker's responsibility is
/// the safety net — *if* a buggy resolver or build pipeline
/// accidentally pulled both alternates into the module set, the
/// tree-shaker must still strip the unreachable one.
///
/// This test sets up the safety-net scenario:
///
/// - `entry.js` imports only `pkg_browser` from `./pkg/b`.
/// - Both `pkg/n.js` (the node alternate) and `pkg/b.js` (the
///   browser alternate) appear in the module set.
/// - Each alternate has matching exports per the closed package
///   contract.
///
/// Expected: `analyze_used_exports` marks only `b.js`'s export as
/// used, and `n.js` lands in `eliminated_modules` because nothing
/// imports it and it's side-effect-free. This is the property
/// "n.js's exports do not leak into the bundle" reduced to its
/// analyzer-observable form.
#[test]
fn conditional_exports_pruning_browser_condition() {
    let modules = fixture(&[
        (
            "/fixture/entry.js",
            "import { pkg_browser } from './pkg/b';\nconsole.log(pkg_browser());\n",
        ),
        (
            "/fixture/pkg/b.js",
            "export const pkg_browser = () => 'browser-impl';\n",
        ),
        (
            "/fixture/pkg/n.js",
            "export const pkg_node = () => 'node-impl';\n",
        ),
    ]);

    let result = analyze_used_exports(&modules).expect("analyze_used_exports");

    // The browser alternate is reached and its export is marked
    // used.
    let browser = PathBuf::from("/fixture/pkg/b.js");
    let browser_used = result
        .used_exports
        .get(&browser)
        .cloned()
        .unwrap_or_default();
    assert!(
        browser_used.contains("pkg_browser"),
        "b.js's `pkg_browser` export must be marked used; got {browser_used:?}",
    );
    assert!(
        !result.eliminated_modules.contains(&browser),
        "b.js must not be eliminated — it's the active conditional alternate",
    );

    // The node alternate is unreachable and side-effect-free, so
    // the safety net catches it: `n.js` lands in
    // `eliminated_modules`. This is the analyzer-observable form
    // of "n.js's exports do not leak into the bundle".
    let node = PathBuf::from("/fixture/pkg/n.js");
    assert!(
        result.eliminated_modules.contains(&node),
        "n.js must be eliminated — nothing imports it under the browser condition; got {:?}",
        result.eliminated_modules,
    );

    snapshot_eq(
        "tree_shaking__conditional_exports_pruning__analysis",
        &result_to_value(&result),
    );
}

// ─── R3 + R4 — bundle-size regression gate + mini-react e2e ─────────────────

/// Run `shake_module` over every module in `modules` and return the
/// total post-shake byte count + a per-module breakdown (sorted by
/// path for stable output). This is the analyzer-observable
/// "bundle size" reduced to its smallest deterministic form: no
/// minification, no scope hoisting, no chunk overhead — just the
/// raw bytes that survive export-level dead-code elimination.
///
/// The number is sufficient as a regression gate: any change that
/// affects what the shaker keeps shows up as a snapshot diff.
fn post_shake_size(
    modules: &[(PathBuf, String)],
    used: &std::collections::HashMap<PathBuf, std::collections::HashSet<String>>,
) -> Value {
    let mut per_module: BTreeMap<String, usize> = BTreeMap::new();
    let mut total: usize = 0;
    for (path, source) in modules {
        let used_for = used.get(path).cloned().unwrap_or_default();
        let shaken = shake_module(source, path, &used_for);
        let bytes = shaken.len();
        per_module.insert(path.to_string_lossy().into_owned(), bytes);
        total += bytes;
    }
    json!({
        "post_shake_bytes_total": total,
        "post_shake_bytes_per_module": per_module,
    })
}

/// **R4 + R3 — mini-react e2e fixture with bundle-size baseline.**
///
/// A self-contained, synthetic-but-realistic React-flavored module
/// graph that exercises the analyzer + shaker as a build pipeline
/// would. Five modules:
///
/// - `react/index.js` — exports `useState`, `useEffect`, and a
///   never-imported `unstable_DebugTracing` helper.
/// - `react-dom/index.js` — exports `render` and the never-
///   imported `unmountComponentAtNode`.
/// - `app/hooks.js` — re-exports only `useState` from `react` (so
///   `useEffect` from react never gets reached on this path; the
///   suite's Slice-4 finding makes that an under-shaken case
///   today, see #1342).
/// - `app/main.js` — pulls `useState` from `./hooks` and `render`
///   from `react-dom`; declares the user component.
/// - `app/devtools.js` — a side-effect-free orphan (e.g. dev-only
///   instrumentation) that the bundler should drop entirely.
///
/// What this test asserts:
///
/// 1. **R4 — realistic fixture.** A snapshot of the analyzer
///    output proves the suite handles a multi-module React-shaped
///    graph end-to-end (re-exports, transitive imports, orphan
///    detection).
/// 2. **R3 — bundle-size regression gate.** A second snapshot
///    pins the post-shake total bytes. Any analyzer or shaker
///    change that flips a module's keep/drop decision moves the
///    number, and the snapshot diff fails CI. `JET_SNAPSHOT_UPDATE=1`
///    accepts intentional changes.
///
/// The fixture exercises the fixed #1342 path: the app imports from a
/// local barrel, and that used signal must thread back to the leaf
/// export while unrelated exports remain removable.
#[test]
fn mini_react_e2e_baseline() {
    let modules = fixture(&[
        // Mini "react" — three exports; one never imported.
        (
            "/fixture/react/index.js",
            "export const useState = (init) => [init, () => {}];\n\
             export const useEffect = (fn) => fn();\n\
             export const unstable_DebugTracing = () => {};\n",
        ),
        // Mini "react-dom" — two exports; one never imported.
        (
            "/fixture/react-dom/index.js",
            "export const render = (el, container) => { container.x = el; };\n\
             export const unmountComponentAtNode = (container) => { container.x = null; };\n",
        ),
        // Re-export barrel — only `useState` flows through to app.
        // Re-export barrel — the used signal should thread back to
        // react/index.js's `useState` without keeping `useEffect` or
        // `unstable_DebugTracing`.
        (
            "/fixture/app/hooks.js",
            "export { useState } from '../react/index';\n",
        ),
        // App entry — pulls a hook + a renderer.
        (
            "/fixture/app/main.js",
            "import { useState } from './hooks';\n\
             import { render } from '../react-dom/index';\n\
             const App = () => {\n  const [n, set] = useState(0);\n  return { tag: 'div', n };\n};\n\
             render(App(), { container: 'root' });\n",
        ),
        // Dev-only orphan — never imported, side-effect-free.
        // Should land in `eliminated_modules`.
        (
            "/fixture/app/devtools.js",
            "export const trace = (msg) => msg;\nexport const profile = (fn) => fn;\n",
        ),
    ]);

    let result = analyze_used_exports_from(&modules, Path::new("/fixture/app/main.js"), None)
        .expect("analyze_used_exports_from");

    // Sanity asserts — pin the analyzer-observable contract that
    // the bundle-size number depends on, so when the snapshot
    // does change a future maintainer can tell the assertion-
    // covered behavior held vs. the bundle-size moved for some
    // other reason.
    let devtools = PathBuf::from("/fixture/app/devtools.js");
    assert!(
        result.eliminated_modules.contains(&devtools),
        "devtools.js must be eliminated — never imported, side-effect-free; got {:?}",
        result.eliminated_modules,
    );
    let hooks = PathBuf::from("/fixture/app/hooks.js");
    let hooks_used = result.used_exports.get(&hooks).cloned().unwrap_or_default();
    assert!(
        hooks_used.contains("useState"),
        "hooks.js's `useState` re-export must be marked used; got {hooks_used:?}",
    );
    let react = PathBuf::from("/fixture/react/index.js");
    let react_used = result.used_exports.get(&react).cloned().unwrap_or_default();
    assert!(
        react_used.contains("useState"),
        "react/index.js's `useState` export must be threaded through hooks.js; got {react_used:?}",
    );
    let react_dom = PathBuf::from("/fixture/react-dom/index.js");
    let react_dom_used = result
        .used_exports
        .get(&react_dom)
        .cloned()
        .unwrap_or_default();
    assert!(
        react_dom_used.contains("render"),
        "react-dom/index.js's `render` export must be marked used by app/main.js; got {react_dom_used:?}",
    );

    snapshot_eq(
        "tree_shaking__mini_react_e2e__analysis",
        &result_to_value(&result),
    );
    snapshot_eq(
        "tree_shaking__mini_react_e2e__bundle_size_baseline",
        &post_shake_size(&modules, &result.used_exports),
    );
}
// CODEGEN-END
