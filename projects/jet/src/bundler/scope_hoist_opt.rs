// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Post-flattening optimizations for scope-hoisted bundles.
//!
//! R4: Cross-module constant inlining — propagate immutable bindings
//!     across module boundaries in the flattened scope.
//! R5: Unified cross-module DCE — eliminate unused exports and dead
//!     functions across the merged scope.
//! R6: sideEffects integration — use `sideEffects: false` from
//!     package.json to identify safe inlining candidates.

use regex::Regex;

use super::scope_hoist::is_id_cont_byte;
use super::CompiledModule;

// ──────────────────────────────────────────────────────────────────────────
// R4: Cross-module constant inlining
// ──────────────────────────────────────────────────────────────────────────

/// Inline cross-module constants in a flattened bundle.
///
/// After `generate_flattened_bundle` produces the merged output, scans for
/// `var _m{i}_NAME = <literal>;` patterns where the initializer is a string,
/// number, or boolean literal. Replaces all references to `_m{i}_NAME` with
/// the literal value. Removes the now-unused `var` declaration line.
///
/// Only applies to bindings that were originally `const` declarations, which
/// are identified by the `_m{i}_` prefix pattern (all flattened const bindings
/// pass through the prefix renaming in `inline_module_body_v2`).
///
/// Literals recognized:
/// - String: `"..."` or `'...'`
/// - Number: integer or float (optionally negative)
/// - Boolean: `true` or `false`
/// - `null`, `undefined`, `void 0`
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn inline_cross_module_constants(code: &str) -> String {
    // Match: var _m{i}_{name} = <literal>;
    // Captures: (1) full var name, (2) literal value
    let re = Regex::new(
        r#"var\s+(_m\d+_[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null|undefined|void\s+0)\s*;"#,
    )
    .unwrap();

    // Phase 1: collect all constant bindings and their literal values
    let mut constants: Vec<(String, String)> = Vec::new();
    for cap in re.captures_iter(code) {
        let var_name = cap[1].to_string();
        let literal = cap[2].to_string();

        // Only inline if the binding has more than 0 read references in the code
        // (otherwise it's dead and will be cleaned up by R5 DCE)
        // Count occurrences of the var name as a standalone identifier
        let count = count_identifier_refs(code, &var_name);
        // count >= 2 means: 1 for the declaration + at least 1 read reference
        if count >= 2 {
            constants.push((var_name, literal));
        }
    }

    if constants.is_empty() {
        return code.to_string();
    }

    let mut result = code.to_string();

    // Phase 2: remove the var declarations FIRST (before replacing identifiers,
    // because `replace_identifier` would also replace the name in the LHS of the
    // declaration, making the removal pattern unmatchable).
    for (var_name, literal) in &constants {
        let decl_pattern = format!("var {}={};", var_name, literal);
        result = result.replace(&decl_pattern, "");
        // Also try with spaces around `=` and before `;`
        let decl_spaced = format!("var {} = {};", var_name, literal);
        result = result.replace(&decl_spaced, "");
    }

    // Phase 3: replace all remaining references with the literal value
    for (var_name, literal) in &constants {
        result = replace_identifier(&result, var_name, literal);
    }

    result
}

/// Count standalone identifier references (not preceded by `.` or part of a
/// longer identifier) in the given code.
fn count_identifier_refs(code: &str, ident: &str) -> usize {
    let b = code.as_bytes();
    let ident_bytes = ident.as_bytes();
    let ident_len = ident_bytes.len();
    let len = b.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        // Skip string literals
        if matches!(b[i], b'"' | b'\'' | b'`') {
            let q = b[i];
            i += 1;
            while i < len {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }
        }

        // Try to match identifier
        if i + ident_len <= len && &b[i..i + ident_len] == ident_bytes {
            // Check word boundaries
            let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
            let next_ok = i + ident_len >= len || !is_id_cont_byte(b[i + ident_len]);
            // Not preceded by '.'
            let not_prop = {
                let mut p = i;
                while p > 0 && matches!(b[p - 1], b' ' | b'\t') {
                    p -= 1;
                }
                p == 0 || b[p - 1] != b'.'
            };
            if prev_ok && next_ok && not_prop {
                count += 1;
                i += ident_len;
                continue;
            }
        }

        i += 1;
    }

    count
}

/// Replace all standalone identifier references (not inside strings, comments,
/// or property accesses) with the given replacement string.
fn replace_identifier(code: &str, ident: &str, replacement: &str) -> String {
    let b = code.as_bytes();
    let ident_bytes = ident.as_bytes();
    let ident_len = ident_bytes.len();
    let len = b.len();
    let mut out = Vec::with_capacity(len + 64);
    let mut i = 0;

    while i < len {
        // Skip string literals
        if matches!(b[i], b'"' | b'\'' | b'`') {
            let q = b[i];
            out.push(b[i]);
            i += 1;
            while i < len {
                if b[i] == b'\\' {
                    out.push(b[i]);
                    i += 1;
                    if i < len {
                        out.push(b[i]);
                        i += 1;
                    }
                    continue;
                }
                out.push(b[i]);
                if b[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    out.push(b[i]);
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                out.push(b[i]);
                i += 1;
                out.push(b[i]);
                i += 1;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    out.push(b[i]);
                    i += 1;
                }
                if i + 1 < len {
                    out.push(b[i]);
                    i += 1;
                    out.push(b[i]);
                    i += 1;
                }
                continue;
            }
        }

        // Try to match identifier at word boundary
        if i + ident_len <= len && &b[i..i + ident_len] == ident_bytes {
            let prev_ok = i == 0 || !is_id_cont_byte(b[i - 1]);
            let next_ok = i + ident_len >= len || !is_id_cont_byte(b[i + ident_len]);
            let not_prop = {
                let mut p = out.len();
                while p > 0 && matches!(out[p - 1], b' ' | b'\t') {
                    p -= 1;
                }
                p == 0 || out[p - 1] != b'.'
            };
            if prev_ok && next_ok && not_prop {
                out.extend_from_slice(replacement.as_bytes());
                i += ident_len;
                continue;
            }
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
}

// ──────────────────────────────────────────────────────────────────────────
// R5: Unified cross-module DCE
// ──────────────────────────────────────────────────────────────────────────

/// Eliminate unused exports and dead variables in a flattened bundle.
///
/// After constant inlining (R4), scans the flattened bundle for:
/// 1. `_m{i}e.NAME` assignment sites. If `_m{i}e.NAME` has zero read
///    references elsewhere in the bundle, remove the assignment statement.
/// 2. Prefixed variable declarations (`var _m{i}_NAME`) with zero remaining
///    references — remove the entire declaration.
///
/// Must compose with existing per-module `dce.rs` pass (which runs before
/// scope hoisting).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn eliminate_unused_exports(code: &str) -> String {
    let mut result = code.to_string();

    // Phase 1: Remove unused _m{i}e.NAME export assignments
    // Match pattern: _m{i}e.NAME = <expr>;
    let export_assign_re = Regex::new(r"(_m\d+e)\.([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=[^=]").unwrap();

    let mut exports_to_remove: Vec<(String, String)> = Vec::new();

    for cap in export_assign_re.captures_iter(&result) {
        let export_obj = cap[1].to_string();
        let export_name = cap[2].to_string();
        let full_ref = format!("{}.{}", export_obj, export_name);

        // Count read references (not the assignment site itself)
        // A "read" is `_m{i}e.NAME` not followed by `=` (or `==`/`===`)
        let read_count = count_export_reads(&result, &full_ref);

        if read_count == 0 {
            exports_to_remove.push((export_obj, export_name));
        }
    }

    // Remove unused export assignment statements
    for (export_obj, export_name) in &exports_to_remove {
        result = remove_export_assignment(&result, export_obj, export_name);
    }

    // Phase 2: Remove unused prefixed variable declarations
    // Match: var _m{i}_NAME = ...;  or  var _m{i}_NAME;
    let prefixed_var_re = Regex::new(r"var\s+(_m\d+_[a-zA-Z_$][a-zA-Z0-9_$]*)").unwrap();

    let mut vars_to_remove: Vec<String> = Vec::new();

    // Collect candidates first from the current state
    for cap in prefixed_var_re.captures_iter(&result) {
        let var_name = cap[1].to_string();
        // Count total references (including the declaration)
        let total_refs = count_identifier_refs(&result, &var_name);
        // If only 1 reference (the declaration itself), the var is unused
        if total_refs <= 1 {
            vars_to_remove.push(var_name);
        }
    }

    // Remove unused variable declarations
    for var_name in &vars_to_remove {
        result = remove_var_declaration(&result, var_name);
    }

    result
}

/// Count read references to an export property like `_m0e.foo`, excluding
/// assignment sites (where it's followed by `=` but not `==`).
fn count_export_reads(code: &str, full_ref: &str) -> usize {
    let b = code.as_bytes();
    let ref_bytes = full_ref.as_bytes();
    let ref_len = ref_bytes.len();
    let len = b.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        // Skip string literals
        if matches!(b[i], b'"' | b'\'' | b'`') {
            let q = b[i];
            i += 1;
            while i < len {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip comments
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }
        }

        // Try to match the export reference
        if i + ref_len <= len && &b[i..i + ref_len] == ref_bytes {
            // Check it's not part of a longer identifier
            let next_ok = i + ref_len >= len || !is_id_cont_byte(b[i + ref_len]);
            if next_ok {
                // Check if this is an assignment (followed by `=` but not `==`)
                let mut j = i + ref_len;
                while j < len && matches!(b[j], b' ' | b'\t') {
                    j += 1;
                }
                let is_assignment = j < len && b[j] == b'=' && (j + 1 >= len || b[j + 1] != b'=');

                if !is_assignment {
                    count += 1;
                }
                i += ref_len;
                continue;
            }
        }

        i += 1;
    }

    count
}

/// Remove an export assignment statement like `_m0e.foo = <expr>;` from code.
fn remove_export_assignment(code: &str, export_obj: &str, export_name: &str) -> String {
    let pattern = format!(
        r"{}\.{}\s*=[^=][^;]*;",
        regex::escape(export_obj),
        regex::escape(export_name)
    );
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(code, "").to_string()
}

/// Remove a `var _m{i}_NAME = <expr>;` or `var _m{i}_NAME;` declaration
/// from code.
fn remove_var_declaration(code: &str, var_name: &str) -> String {
    // Match: var NAME = <anything up to ;>;  or  var NAME;
    let pattern = format!(r"var\s+{}\s*(?:=[^;]*)?;", regex::escape(var_name));
    let re = Regex::new(&pattern).unwrap();
    re.replace_all(code, "").to_string()
}

// ──────────────────────────────────────────────────────────────────────────
// R6: sideEffects integration
// ──────────────────────────────────────────────────────────────────────────

/// Check if a compiled module is side-effect-free based on its source path.
///
/// Uses the `sideEffects` field from the owning package's `package.json`:
/// - `sideEffects: false` → module is side-effect-free (safe to inline)
/// - `sideEffects: true` or absent → check code heuristically
/// - `sideEffects: ["*.css", ...]` → side-effect-free unless path matches a glob
///
/// Modules with side effects must NOT be inlined during scope hoisting —
/// they retain their wrapper boundary to preserve execution order.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_side_effect_free(module: &CompiledModule) -> bool {
    use super::tree_shake::{has_side_effects, module_has_side_effects, read_package_side_effects};

    // Try to find the owning package's node_modules directory.
    // Walk up from the module path to find `node_modules/{package}/package.json`.
    let module_path = &module.path;
    if let Some(nm_and_pkg) = find_package_info(module_path) {
        let (node_modules_dir, package_name) = nm_and_pkg;
        let decl = read_package_side_effects(&node_modules_dir, &package_name);
        !module_has_side_effects(&module.code, module_path, &decl)
    } else {
        // Not inside node_modules — use heuristic code analysis.
        // Project source files are conservatively assumed to have side effects
        // unless analysis says otherwise.
        !has_side_effects(&module.code)
    }
}

/// Extract the `node_modules` directory path and the package name from a
/// module's absolute path.
///
/// For example:
///   `/project/node_modules/react/cjs/react.production.min.js`
///   → `("/project/node_modules", "react")`
///
///   `/project/node_modules/@scope/pkg/index.js`
///   → `("/project/node_modules", "@scope/pkg")`
fn find_package_info(module_path: &std::path::Path) -> Option<(std::path::PathBuf, String)> {
    let path_str = module_path.to_string_lossy();

    // Find the last `node_modules/` in the path
    let nm_marker = "node_modules/";
    let nm_pos = path_str.rfind(nm_marker)?;

    let node_modules_dir = std::path::PathBuf::from(&path_str[..nm_pos + nm_marker.len() - 1]);
    let after_nm = &path_str[nm_pos + nm_marker.len()..];

    // Extract package name: either `@scope/name` or `name`
    let package_name = if after_nm.starts_with('@') {
        // Scoped package: @scope/name
        let parts: Vec<&str> = after_nm.splitn(3, '/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            return None;
        }
    } else {
        // Regular package: name
        after_nm.split('/').next()?.to_string()
    };

    Some((node_modules_dir, package_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_module(path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            path: PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // R4: inline_cross_module_constants
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_inline_cross_module_constants_string() {
        // A const string binding `_m1_MODE = "production"` is referenced
        // in a conditional — after inlining, the reference is replaced
        // with the literal value.
        let code = r#"var _m1_MODE = "production";
if (_m1_MODE !== "production") { debugSetup(); }
console.log(_m1_MODE);"#;

        let result = inline_cross_module_constants(code);

        // The literal "production" should replace all references
        assert!(
            !result.contains("_m1_MODE"),
            "all references to _m1_MODE should be inlined, got: {}",
            result
        );
        // The inlined literal should appear in the conditional
        assert!(
            result.contains(r#""production" !== "production""#),
            "conditional should have inlined literal, got: {}",
            result
        );
        // The var declaration line should be removed
        assert!(
            !result.contains("var "),
            "var declaration should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_number() {
        // A const number binding is propagated to all usage sites.
        let code = "var _m0_MAX_SIZE = 1024;\nvar _m0_arr = new Array(_m0_MAX_SIZE);\nconsole.log(_m0_MAX_SIZE);";

        let result = inline_cross_module_constants(code);

        assert!(
            !result.contains("_m0_MAX_SIZE"),
            "all references to _m0_MAX_SIZE should be inlined, got: {}",
            result
        );
        assert!(
            result.contains("new Array(1024)"),
            "number literal should be propagated, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_boolean() {
        let code = "var _m2_DEV = false;\nif (_m2_DEV) { enableDevTools(); }\nvar _m2_x = _m2_DEV;";

        let result = inline_cross_module_constants(code);

        assert!(
            !result.contains("_m2_DEV"),
            "_m2_DEV should be inlined, got: {}",
            result
        );
        assert!(
            result.contains("if (false)"),
            "boolean literal should be propagated, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_no_inline_non_literal() {
        // Non-literal initializers (function calls, object expressions) must
        // NOT be inlined.
        let code = "var _m0_config = getConfig();\nconsole.log(_m0_config);";

        let result = inline_cross_module_constants(code);

        // Should remain unchanged — getConfig() is not a literal
        assert!(
            result.contains("_m0_config"),
            "non-literal binding should not be inlined, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_single_ref_not_inlined() {
        // A binding with only the declaration (no read references) should
        // NOT be inlined (it's dead code, handled by R5 DCE instead).
        let code = "var _m0_UNUSED = 42;\nconsole.log('hello');";

        let result = inline_cross_module_constants(code);

        // Only 1 reference (the declaration itself) — should not be inlined
        assert!(
            result.contains("_m0_UNUSED"),
            "unused binding should not be inlined by R4, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_preserves_strings() {
        // References inside string literals should NOT be replaced
        let code = r#"var _m0_NAME = "foo";
var _m0_msg = "the value of _m0_NAME is " + _m0_NAME;"#;

        let result = inline_cross_module_constants(code);

        // The string content should be preserved
        assert!(
            result.contains("\"the value of _m0_NAME is \""),
            "string content should not be modified, got: {}",
            result
        );
        // The identifier reference outside the string should be replaced
        assert!(
            result.contains("+ \"foo\""),
            "identifier reference should be replaced with literal, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_cross_module_constants_null_undefined() {
        let code = "var _m0_val = null;\nif (_m0_val) { doSomething(); }\nvar _m0_x = _m0_val;";

        let result = inline_cross_module_constants(code);

        assert!(
            !result.contains("_m0_val"),
            "null literal should be inlined, got: {}",
            result
        );
        assert!(
            result.contains("if (null)"),
            "null should replace the reference, got: {}",
            result
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R5: eliminate_unused_exports
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_eliminate_unused_exports() {
        // An export assignment `_m0e.unusedFn = ...` with zero read references
        // in the bundle should be removed entirely.
        let code = r#"var _m0e = _m0.exports;
_m0e.usedFn = function() { return 42; };
_m0e.unusedFn = function() { return 99; };
var _m1_result = _m0e.usedFn();"#;

        let result = eliminate_unused_exports(code);

        // usedFn is referenced → must survive
        assert!(
            result.contains("_m0e.usedFn"),
            "used export should survive, got: {}",
            result
        );
        // unusedFn has no read reference → should be removed
        assert!(
            !result.contains("unusedFn"),
            "unused export should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_keeps_read_refs() {
        // Both exports are read — neither should be removed.
        let code = r#"_m0e.foo = 1;
_m0e.bar = 2;
var _m1_x = _m0e.foo + _m0e.bar;"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0e.foo = 1"),
            "foo export should survive (has read ref), got: {}",
            result
        );
        assert!(
            result.contains("_m0e.bar = 2"),
            "bar export should survive (has read ref), got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars() {
        // A prefixed var `_m0_helper` with no remaining references after DCE
        // should be removed.
        let code = r#"var _m0_used = 42;
var _m0_helper = function() { return 99; };
console.log(_m0_used);"#;

        let result = eliminate_unused_exports(code);

        // _m0_used has a reference → survive
        assert!(
            result.contains("_m0_used"),
            "used var should survive, got: {}",
            result
        );
        // _m0_helper has only the declaration → removed
        assert!(
            !result.contains("_m0_helper"),
            "unused prefixed var should be removed, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_prefixed_vars_with_refs() {
        // A prefixed var that IS referenced should NOT be removed.
        let code = r#"var _m0_count = 0;
_m0_count++;
console.log(_m0_count);"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0_count"),
            "referenced prefixed var should survive, got: {}",
            result
        );
    }

    #[test]
    fn test_eliminate_unused_exports_comparison_not_counted_as_assignment() {
        // `_m0e.foo === "bar"` is a read (comparison), not an assignment.
        // The export should survive if it has comparisons as reads.
        let code = r#"_m0e.foo = "bar";
if (_m0e.foo === "bar") { doSomething(); }"#;

        let result = eliminate_unused_exports(code);

        assert!(
            result.contains("_m0e.foo"),
            "export with comparison reads should survive, got: {}",
            result
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R6: is_side_effect_free
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_side_effect_free_pure_module() {
        // A module with only declarations (var/function/class) is
        // considered side-effect-free by the heuristic code analysis.
        // Note: `exports.X = ...` lines are conservatively treated as
        // side-effectful by `has_side_effects` since they don't start
        // with a recognized declaration keyword.
        let module = make_module(
            "/project/src/utils.js",
            "function add(a, b) { return a + b; }\nvar PI = 3.14;",
        );
        assert!(
            is_side_effect_free(&module),
            "pure declaration-only module should be side-effect-free"
        );
    }

    #[test]
    fn test_side_effect_cjs_exports_considered_effectful() {
        // CJS `exports.xxx = ...` is treated as a side effect by the
        // heuristic since it doesn't start with a declaration keyword.
        // This is conservative but correct for non-node_modules code.
        let module = make_module(
            "/project/src/lib.js",
            "exports.add = function(a, b) { return a + b; };",
        );
        assert!(
            !is_side_effect_free(&module),
            "CJS exports assignment should be conservatively treated as side-effectful"
        );
    }

    #[test]
    fn test_side_effect_module_not_flattened() {
        // A module with top-level side effects (DOM manipulation, global writes)
        // should NOT be considered side-effect-free.
        let module = make_module(
            "/project/src/init.js",
            "document.title = 'Hello';\nexports.ready = true;",
        );
        assert!(
            !is_side_effect_free(&module),
            "module with DOM side effects should NOT be side-effect-free"
        );
    }

    #[test]
    fn test_side_effect_module_global_assignment() {
        // Global variable assignment is a side effect.
        let module = make_module(
            "/project/src/polyfill.js",
            "window.Promise = require('./promise');\nexports.done = true;",
        );
        assert!(
            !is_side_effect_free(&module),
            "module with global assignment should NOT be side-effect-free"
        );
    }

    #[test]
    fn test_side_effect_free_const_only() {
        // A module with only var/const declarations is side-effect-free.
        // Note: `exports.MODE = MODE` would be treated as a side effect
        // by the heuristic, so we use pure declarations only.
        let module = make_module(
            "/project/src/constants.js",
            "var MODE = 'production';\nconst VERSION = '1.0';",
        );
        assert!(
            is_side_effect_free(&module),
            "const+var declaration module should be side-effect-free"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Helper: find_package_info
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_find_package_info_regular() {
        let path = PathBuf::from("/project/node_modules/react/cjs/react.production.min.js");
        let result = find_package_info(&path);
        assert!(result.is_some());
        let (nm_dir, pkg_name) = result.unwrap();
        assert_eq!(nm_dir, PathBuf::from("/project/node_modules"));
        assert_eq!(pkg_name, "react");
    }

    #[test]
    fn test_find_package_info_scoped() {
        let path = PathBuf::from("/project/node_modules/@babel/core/lib/index.js");
        let result = find_package_info(&path);
        assert!(result.is_some());
        let (nm_dir, pkg_name) = result.unwrap();
        assert_eq!(nm_dir, PathBuf::from("/project/node_modules"));
        assert_eq!(pkg_name, "@babel/core");
    }

    #[test]
    fn test_find_package_info_not_in_node_modules() {
        let path = PathBuf::from("/project/src/utils.js");
        let result = find_package_info(&path);
        assert!(result.is_none());
    }

    // ──────────────────────────────────────────────────────────────────
    // Helper: count_identifier_refs
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_count_identifier_refs_basic() {
        let code = "var _m0_x = 1; console.log(_m0_x); return _m0_x;";
        assert_eq!(count_identifier_refs(code, "_m0_x"), 3);
    }

    #[test]
    fn test_count_identifier_refs_skips_strings() {
        let code = r#"var _m0_x = 1; var s = "_m0_x"; return _m0_x;"#;
        // 2 real refs (declaration + return), 1 inside string (skipped)
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }

    #[test]
    fn test_count_identifier_refs_skips_property_access() {
        let code = "var _m0_x = 1; obj._m0_x = 2; return _m0_x;";
        // obj._m0_x is preceded by `.` — should be skipped
        // Only declaration + return = 2
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }

    #[test]
    fn test_count_identifier_refs_skips_comments() {
        let code = "var _m0_x = 1; // _m0_x is defined here\nreturn _m0_x;";
        // Comment reference is skipped
        assert_eq!(count_identifier_refs(code, "_m0_x"), 2);
    }
}
// CODEGEN-END
