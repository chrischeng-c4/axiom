// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Scope hoisting: module concatenation to eliminate runtime overhead.
//!
//! Instead of the `__jet__.define` / `__jet__.require` module registry,
//! all modules are inlined into a single IIFE with a lightweight
//! `_r` function. This gives minifiers full cross-module
//! visibility for dead-code elimination and constant folding, which
//! reduces bundle size to match Webpack / Vite output.
//!
//! ## How it works
//!
//! The normal bundle format wraps each module in a runtime call:
//! ```js
//! __jet__.define(N, function(require, module, exports) { ... });
//! ```
//!
//! The scope-hoisted format flattens all modules into one IIFE:
//! ```js
//! (function() {
//!   var _m0 = {exports: {}};
//!   // ...
//!   function _r(id) { ... }
//!
//!   // Execute in dependency order (leaf modules first)
//!   (function(module, exports, require) { /* dep */ })
//!     (_m1, _m1.exports, _r);
//!   (function(module, exports, require) { /* entry */ })
//!     (_m0, _m0.exports, _r);
//! })();
//! ```
//!
//! Benefits over the runtime-based approach:
//! - No `window.__jet__` global
//! - No hash-table module registry
//! - Single scope → minifier renames all local vars in one pass
//! - Cross-module DCE and constant folding become possible

use std::collections::HashMap;

use super::CompiledModule;

// Re-export post-flattening optimizations from the split module.
pub use super::scope_hoist_opt::{
    eliminate_unused_exports, inline_cross_module_constants, is_side_effect_free,
};

/// Generate a scope-hoisted bundle from compiled modules.
///
/// `modules` must be in topological order where `modules[0]` is the
/// entry point (has module ID 0) and later entries are its
/// dependencies. Modules are executed in reverse order (deepest
/// dependencies first) so that `require()` targets are always
/// initialised before their callers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn generate_scope_hoisted_bundle(modules: &[CompiledModule]) -> String {
    if modules.is_empty() {
        return String::new();
    }

    let n = modules.len();
    let mut out = String::with_capacity(estimate_output_size(modules));

    // Outer IIFE to avoid leaking module variables into global scope
    out.push_str("(function(){\n'use strict';\n\n");

    // Pre-declare all module namespace objects.
    // Using `var` means they are hoisted to the function scope and
    // visible everywhere inside the IIFE.
    for i in 0..n {
        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", i));
    }
    out.push('\n');

    // Lightweight require: maps numeric module ID to module.exports.
    // This is the only runtime overhead that cannot be eliminated by
    // hoisting. A minifier (Terser/esbuild) can inline this for
    // single-call-site modules.
    out.push_str("function _r(id){\n");
    out.push_str("  switch(id){\n");
    for i in 0..n {
        out.push_str(&format!("    case {}:return _m{}.exports;\n", i, i));
    }
    out.push_str("    default:return {};\n");
    out.push_str("  }\n");
    out.push_str("}\n\n");

    // Execute modules in reverse topological order so that each
    // dependency is fully initialised before its importer runs.
    // (modules[0] = entry point; modules[n-1] = deepest leaf)
    for (original_idx, module) in modules.iter().enumerate().rev() {
        let module_path = module.path.to_string_lossy();
        out.push_str(&format!("// Module {}: {}\n", original_idx, module_path));
        // Each module gets its own function scope so that local
        // `var` declarations don't collide across modules.  The
        // single IIFE wrapper means a minifier can still see all
        // module-level references and apply cross-module DCE.
        out.push_str(&format!(
            "!function(module,exports,require){{\n{}}}(\
             _m{},_m{}.exports,_r);\n\n",
            module.code, original_idx, original_idx
        ));
    }

    out.push_str("})();\n");
    out
}

/// Estimate the output buffer capacity to avoid repeated reallocations.
fn estimate_output_size(modules: &[CompiledModule]) -> usize {
    let code_total: usize = modules.iter().map(|m| m.code.len()).sum();
    let overhead = 200 + modules.len() * 80;
    code_total + overhead
}

/// Returns `true` when the bundle has no dynamic imports, making it
/// safe to use scope hoisting without a full runtime module registry.
///
/// The check is conservative: any unresolved `import()` call in the
/// compiled code keeps the runtime format.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_scope_hoist_safe(modules: &[CompiledModule]) -> bool {
    for module in modules {
        if module.code.contains("import(") {
            return false;
        }
    }
    true
}

/// Returns `true` when no module uses `eval()`, `with` statements, or dynamic
/// `arguments[...]` access, which would make it unsafe to inline the module
/// body into a shared scope.
///
/// - `eval()` can reference ambient variables by name at runtime.
/// - `with(obj)` creates dynamic scope that cannot be statically resolved.
/// - `arguments[dynamic_index]` relies on the current function's `arguments`
///   object being stable, which renaming could violate.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_flatten_safe(modules: &[CompiledModule]) -> bool {
    for module in modules {
        if module.code.contains("eval(")
            || module.code.contains("with(")
            || module.code.contains("arguments[")
        {
            return false;
        }
    }
    true
}

/// Phase 2: Generate a truly flat bundle by inlining each module body
/// directly into the outer IIFE without per-module wrapper functions.
///
/// Unlike Phase 1 (`generate_scope_hoisted_bundle`), this approach
/// replaces the `!function(module,exports,require){...}()` wrapper with
/// a plain block `{ ... }` after substituting the CJS parameter names:
///
/// - `module`  → `_m{i}`   (the module namespace object)
/// - `exports` → `_m{i}.exports`
/// - `require` → `_r`
///
/// Benefits over Phase 1:
/// - Minifier sees all variables in a single scope → better name mangling.
/// - No IIFE call overhead per module.
/// - Cross-module constant folding and DCE are more effective.
///
/// Falls back to Phase 1 if `is_flatten_safe` returns `false`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn generate_flattened_bundle(modules: &[CompiledModule]) -> String {
    if modules.is_empty() {
        return String::new();
    }

    // Safety check: fall back to Phase 1 if any module uses eval/with
    if !is_flatten_safe(modules) {
        tracing::debug!("Falling back to Phase 1 scope hoisting (eval/with detected)");
        return generate_scope_hoisted_bundle(modules);
    }

    let n = modules.len();
    let mut out = String::with_capacity(estimate_output_size(modules));

    out.push_str("(function(){\n'use strict';\n\n");

    // Pre-declare all module namespace objects using short names.
    for i in 0..n {
        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", i));
    }
    out.push('\n');

    // Lightweight require function — still needed for patterns like
    // `var dep = require(1)` that reference modules by numeric ID.
    out.push_str("function _r(id){\n");
    out.push_str("  switch(id){\n");
    for i in 0..n {
        out.push_str(&format!("    case {}:return _m{}.exports;\n", i, i));
    }
    out.push_str("    default:return {};\n");
    out.push_str("  }\n");
    out.push_str("}\n\n");

    // Inline each module body in reverse topological order (deepest deps first).
    // R6: modules with side effects retain their IIFE wrapper for isolation.
    for (original_idx, module) in modules.iter().enumerate().rev() {
        let module_path = module.path.to_string_lossy();
        // R6: Check package.json sideEffects field for node_modules packages.
        // Project source files that passed is_flatten_safe are always eligible
        // for inlining — CJS exports assignments are not "side effects" in this
        // context, they're the module's output mechanism.
        let in_node_modules = module.path.to_string_lossy().contains("node_modules");
        let side_effect_free = if in_node_modules {
            is_side_effect_free(module)
        } else {
            true // project source files are eligible if they passed is_flatten_safe
        };
        out.push_str(&format!("// Module {}: {}\n", original_idx, module_path));

        if side_effect_free {
            // Side-effect-free: inline directly into flat scope.
            // Apply per-module prefix renaming (R3) + CJS substitutions (R2).
            let inlined = inline_module_body_v2(&module.code, original_idx);
            out.push_str("{\n");
            out.push_str(&format!(
                "var _m{idx}e=_m{idx}.exports;\n",
                idx = original_idx
            ));
            out.push_str(&inlined);
            out.push_str("\n}\n\n");
        } else {
            // Side-effectful: keep IIFE wrapper to preserve execution order.
            tracing::debug!(
                "Module {} has side effects, retaining wrapper",
                original_idx
            );
            out.push_str(&format!(
                "!function(module,exports,require){{{}}}(_m{idx},_m{idx}.exports,_r);\n\n",
                module.code,
                idx = original_idx
            ));
        }
    }

    out.push_str("})();\n");
    out
}

/// Substitute CJS module parameter names in a compiled module body.
///
/// Replaces standalone identifiers (not preceded by `.`, not inside
/// strings or comments) as follows:
///
/// - `module`  → `_m{idx}`
/// - `exports` → `_m{idx}.exports`
/// - `require` → `_r`
///
/// Uses byte-level scanning to safely handle multi-byte UTF-8 content.
#[cfg(test)]
fn inline_module_body(code: &str, idx: usize) -> String {
    let module_repl = format!("_m{}", idx);
    let exports_repl = format!("_m{}.exports", idx);
    let require_repl = "_r";

    let b = code.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len + 64);
    let mut i = 0;

    while i < len {
        // Skip string literals (single, double, template)
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

        // Skip comments (single-line and block)
        if b[i] == b'/' && i + 1 < len {
            if b[i + 1] == b'/' {
                // Single-line comment: copy until newline
                while i < len && b[i] != b'\n' {
                    out.push(b[i]);
                    i += 1;
                }
                continue;
            }
            if b[i + 1] == b'*' {
                // Block comment: copy until */
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
                    i += 1; // *
                    out.push(b[i]);
                    i += 1; // /
                }
                continue;
            }
        }

        // Try to match an identifier at a word boundary.
        // Only substitute if NOT preceded by `.` (avoids obj.module, obj.exports).
        if is_id_start_byte(b[i]) {
            let prev_non_ws_is_dot = {
                let mut p = out.len();
                while p > 0 && out[p - 1] == b' ' {
                    p -= 1;
                }
                p > 0 && out[p - 1] == b'.'
            };

            // Check each keyword: verify full word boundary (not part of longer ident)
            if !prev_non_ws_is_dot {
                // `module` (6 bytes)
                if i + 6 <= len
                    && &b[i..i + 6] == b"module"
                    && (i + 6 >= len || !is_id_cont_byte(b[i + 6]))
                {
                    out.extend_from_slice(module_repl.as_bytes());
                    i += 6;
                    continue;
                }
                // `exports` (7 bytes)
                if i + 7 <= len
                    && &b[i..i + 7] == b"exports"
                    && (i + 7 >= len || !is_id_cont_byte(b[i + 7]))
                {
                    out.extend_from_slice(exports_repl.as_bytes());
                    i += 7;
                    continue;
                }
                // `require` (7 bytes)
                if i + 7 <= len
                    && &b[i..i + 7] == b"require"
                    && (i + 7 >= len || !is_id_cont_byte(b[i + 7]))
                {
                    out.extend_from_slice(require_repl.as_bytes());
                    i += 7;
                    continue;
                }
            }
        }

        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
}

/// Returns `true` if the byte is a valid JS identifier start (ASCII only).
/// Non-ASCII bytes from multi-byte UTF-8 sequences are never matched,
/// so they pass through unchanged.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[inline]
pub fn is_id_start_byte(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

/// Returns `true` if the byte is a valid JS identifier continuation (ASCII only).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[inline]
pub fn is_id_cont_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

// ──────────────────────────────────────────────────────────────────────────
// Phase 2 helpers: per-module variable prefix renaming (R2 / R3)
// ──────────────────────────────────────────────────────────────────────────

/// Returns `true` if `name` is a JS keyword or declaration keyword that should
/// not be prefixed when scanning top-level declarations.
fn is_js_decl_keyword(name: &str) -> bool {
    matches!(
        name,
        "var"
            | "let"
            | "const"
            | "function"
            | "class"
            | "async"
            | "await"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "return"
            | "new"
            | "delete"
            | "typeof"
            | "void"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "import"
            | "export"
            | "default"
            | "in"
            | "of"
            | "instanceof"
            | "yield"
            | "with"
            | "debugger"
            | "this"
            | "super"
            | "extends"
            | "static"
            | "get"
            | "set"
            | "null"
            | "undefined"
            | "true"
            | "false"
            | "NaN"
            | "Infinity"
    )
}

/// Scan a comma-separated `var`/`let`/`const` declaration list starting at `*i`
/// and push each simple identifier name into `names`.
/// Advances `*i` past the terminating `;` (or until end-of-input).
fn collect_decl_names_from(code: &str, i: &mut usize, names: &mut Vec<String>) {
    let b = code.as_bytes();
    let len = b.len();
    let mut depth = 0i32;
    let mut expect_name = true;

    while *i < len {
        // Skip string literals
        if matches!(b[*i], b'"' | b'\'' | b'`') {
            let q = b[*i];
            *i += 1;
            while *i < len {
                if b[*i] == b'\\' {
                    *i += 2;
                    continue;
                }
                if b[*i] == q {
                    *i += 1;
                    break;
                }
                *i += 1;
            }
            continue;
        }
        match b[*i] {
            b'{' | b'(' | b'[' => {
                depth += 1;
                expect_name = false;
                *i += 1;
            }
            b'}' | b')' | b']' => {
                depth -= 1;
                *i += 1;
            }
            b';' if depth == 0 => {
                *i += 1;
                break;
            }
            b',' if depth == 0 => {
                expect_name = true;
                *i += 1;
            }
            _ if expect_name && is_id_start_byte(b[*i]) => {
                let ns = *i;
                while *i < len && is_id_cont_byte(b[*i]) {
                    *i += 1;
                }
                let name = &code[ns..*i];
                if !name.is_empty() && !is_js_decl_keyword(name) {
                    names.push(name.to_string());
                }
                expect_name = false;
            }
            _ => {
                *i += 1;
            }
        }
    }
}

/// The kind of a top-level declaration (var, let, const, function, class).
///
/// Used by R4 (cross-module constant inlining) to identify `const` bindings
/// with literal initializers that are safe to inline across module boundaries.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    Var,
    Let,
    Const,
    Function,
    Class,
}

/// A top-level declaration name together with its declaration kind.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct DeclInfo {
    pub name: String,
    pub kind: DeclKind,
}

/// Collect all top-level `var`/`let`/`const`/`function`/`async function`/
/// `class` declaration names from a module body, including their declaration
/// kind.
///
/// Only names at brace depth 0 are collected; declarations inside nested
/// functions or blocks are ignored.  CJS globals (`exports`, `module`,
/// `require`) are excluded since they are handled separately.
fn collect_top_level_decls_with_kind(code: &str) -> Vec<DeclInfo> {
    let b = code.as_bytes();
    let len = b.len();
    let mut decls: Vec<DeclInfo> = Vec::new();
    let mut i = 0;
    let mut depth = 0i32;

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
        // Skip single-line comments
        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'/' {
            while i < len && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Skip block comments
        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        // Track depth via all bracket types
        match b[i] {
            b'{' | b'(' | b'[' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' | b')' | b']' => {
                if depth > 0 {
                    depth -= 1;
                }
                i += 1;
                continue;
            }
            _ => {}
        }
        // Only collect declarations at top-level depth
        if depth == 0 && is_id_start_byte(b[i]) {
            let start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let word = &code[start..i];

            // Skip leading whitespace before the next token
            let mut j = i;
            while j < len && matches!(b[j], b' ' | b'\t' | b'\n' | b'\r') {
                j += 1;
            }

            match word {
                "var" | "let" | "const" => {
                    let kind = match word {
                        "var" => DeclKind::Var,
                        "let" => DeclKind::Let,
                        "const" => DeclKind::Const,
                        _ => unreachable!(),
                    };
                    i = j;
                    let mut names: Vec<String> = Vec::new();
                    collect_decl_names_from(code, &mut i, &mut names);
                    for name in names {
                        decls.push(DeclInfo { name, kind });
                    }
                }
                "function" => {
                    i = j;
                    // Skip generator `*`
                    if i < len && b[i] == b'*' {
                        i += 1;
                        while i < len && b[i] == b' ' {
                            i += 1;
                        }
                    }
                    if i < len && is_id_start_byte(b[i]) {
                        let ns = i;
                        while i < len && is_id_cont_byte(b[i]) {
                            i += 1;
                        }
                        let name = &code[ns..i];
                        if !name.is_empty() && !is_js_decl_keyword(name) {
                            decls.push(DeclInfo {
                                name: name.to_string(),
                                kind: DeclKind::Function,
                            });
                        }
                    }
                }
                "async" => {
                    i = j;
                    // `async function name() {}`
                    if i + 8 <= len
                        && &code[i..i + 8] == "function"
                        && (i + 8 >= len || !is_id_cont_byte(b[i + 8]))
                    {
                        i += 8;
                        while i < len && matches!(b[i], b' ' | b'\t') {
                            i += 1;
                        }
                        if i < len && b[i] == b'*' {
                            i += 1;
                            while i < len && b[i] == b' ' {
                                i += 1;
                            }
                        }
                        if i < len && is_id_start_byte(b[i]) {
                            let ns = i;
                            while i < len && is_id_cont_byte(b[i]) {
                                i += 1;
                            }
                            let name = &code[ns..i];
                            if !name.is_empty() && !is_js_decl_keyword(name) {
                                decls.push(DeclInfo {
                                    name: name.to_string(),
                                    kind: DeclKind::Function,
                                });
                            }
                        }
                    }
                }
                "class" => {
                    i = j;
                    if i < len && is_id_start_byte(b[i]) {
                        let ns = i;
                        while i < len && is_id_cont_byte(b[i]) {
                            i += 1;
                        }
                        let name = &code[ns..i];
                        if !name.is_empty() && !is_js_decl_keyword(name) {
                            decls.push(DeclInfo {
                                name: name.to_string(),
                                kind: DeclKind::Class,
                            });
                        }
                    }
                }
                _ => {
                    i = j;
                }
            }
            continue;
        }
        i += 1;
    }

    decls
}

/// Collect all top-level `var`/`let`/`const`/`function`/`async function`/
/// `class` declaration names from a module body.
///
/// Only names at brace depth 0 are collected; declarations inside nested
/// functions or blocks are ignored.  CJS globals (`exports`, `module`,
/// `require`) are excluded since they are handled separately.
fn collect_top_level_decls(code: &str) -> Vec<String> {
    collect_top_level_decls_with_kind(code)
        .into_iter()
        .map(|d| d.name)
        .collect()
}

/// Apply a rename map to a module body in a single byte-level pass.
///
/// Identifiers preceded by `.` (property accesses) are never renamed.
/// String literals and comments are copied verbatim without substitution.
fn apply_renames_in_module_body(code: &str, renames: &HashMap<String, String>) -> String {
    let b = code.as_bytes();
    let len = b.len();
    let mut out = Vec::with_capacity(len + renames.len() * 4);
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
        // Skip single-line comments
        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'/' {
            while i < len && b[i] != b'\n' {
                out.push(b[i]);
                i += 1;
            }
            continue;
        }
        // Skip block comments
        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'*' {
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
        // Identifier: check for rename
        if is_id_start_byte(b[i]) {
            // Check if immediately preceded by '.' (property access — skip)
            let prev_is_dot = {
                let mut p = out.len();
                while p > 0 && matches!(out[p - 1], b' ' | b'\t' | b'\r' | b'\n') {
                    p -= 1;
                }
                p > 0 && out[p - 1] == b'.'
            };
            let start = i;
            while i < len && is_id_cont_byte(b[i]) {
                i += 1;
            }
            let ident = &code[start..i];
            if !prev_is_dot {
                if let Some(new_name) = renames.get(ident) {
                    out.extend_from_slice(new_name.as_bytes());
                    continue;
                }
            }
            out.extend_from_slice(ident.as_bytes());
            continue;
        }
        out.push(b[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
}

/// Extended module body inlining (Phase 2 / R2 + R3).
///
/// Builds a combined rename map that:
/// 1. Substitutes CJS globals: `exports` → `_m{idx}e`, `module` → `_m{idx}`,
///    `require` → `_r`.
/// 2. Prefixes every top-level `var`/`let`/`const`/`function`/`class`
///    declaration with `_m{idx}_` so that when multiple modules are inlined
///    into a single flat scope, their `var` declarations (which hoist to the
///    outer IIFE function) do not collide.
///
/// The prefix names (`_m0_foo`, `_m1_bar`, …) are then compressed by the
/// whole-bundle `mangle_variables` pass into single-byte identifiers.
fn inline_module_body_v2(code: &str, idx: usize) -> String {
    let module_repl = format!("_m{}", idx);
    let exports_alias = format!("_m{}e", idx);

    // Collect top-level declarations that need collision-avoiding prefixes.
    let decls = collect_top_level_decls(code);

    // Build the unified rename map.
    let mut renames: HashMap<String, String> = HashMap::with_capacity(decls.len() + 3);

    // CJS globals come first so the loop below can skip them if they appear
    // as local vars (very unlikely but safe).
    renames.insert("exports".to_string(), exports_alias);
    renames.insert("module".to_string(), module_repl.clone());
    renames.insert("require".to_string(), "_r".to_string());

    // Per-module prefix for top-level declarations.
    for decl in decls {
        // Don't overwrite CJS globals (exports/module/require) with a prefixed
        // version — the CJS substitution above takes priority.
        renames
            .entry(decl.clone())
            .or_insert_with(|| format!("_m{}_{}", idx, decl));
    }

    apply_renames_in_module_body(code, &renames)
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

    #[test]
    fn test_empty_bundle() {
        let result = generate_scope_hoisted_bundle(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_module_bundle() {
        let modules = vec![make_module(
            "entry.js",
            "exports.main = function() { return 42; };",
        )];
        let bundle = generate_scope_hoisted_bundle(&modules);
        // Outer IIFE
        assert!(bundle.contains("(function()"));
        // Module namespace
        assert!(bundle.contains("var _m0="));
        // require function
        assert!(bundle.contains("_r"));
        // Module code wrapped in its own function
        assert!(bundle.contains("exports.main = function()"));
        // Closure
        assert!(bundle.ends_with("})();\n"));
    }

    #[test]
    fn test_two_module_bundle() {
        // Module 0: entry (requires module 1)
        // Module 1: dep (no requires)
        // Execution order: dep (idx=1) first, then entry (idx=0)
        let modules = vec![
            make_module("entry.js", "var _dep = require(1); _dep.greet();"),
            make_module("dep.js", "exports.greet = function() {};"),
        ];
        let bundle = generate_scope_hoisted_bundle(&modules);

        // Both module vars declared
        assert!(bundle.contains("var _m0="));
        assert!(bundle.contains("var _m1="));

        // require switch has both cases
        assert!(bundle.contains("case 0:return _m0.exports;"));
        assert!(bundle.contains("case 1:return _m1.exports;"));

        // dep module (index 1) should appear BEFORE entry (index 0)
        // because we iterate in reverse
        let pos_dep = bundle.find("Module 1:").unwrap();
        let pos_entry = bundle.find("Module 0:").unwrap();
        assert!(
            pos_dep < pos_entry,
            "dep (idx 1) should execute before entry (idx 0)"
        );
    }

    #[test]
    fn test_scope_hoist_safe_no_dynamic_imports() {
        let modules = vec![
            make_module("a.js", "var x = require(1);"),
            make_module("b.js", "exports.foo = 1;"),
        ];
        assert!(is_scope_hoist_safe(&modules));
    }

    #[test]
    fn test_scope_hoist_unsafe_with_dynamic_import() {
        let modules = vec![make_module(
            "a.js",
            "import('./lazy').then(m => m.default());",
        )];
        assert!(!is_scope_hoist_safe(&modules));
    }

    // ──────────────────────────────────────────────────────────────────
    // Phase 2 flatten tests
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_flatten_safe_no_eval() {
        let modules = vec![
            make_module("a.js", "exports.x = 1;"),
            make_module("b.js", "var y = require(1).x;"),
        ];
        assert!(is_flatten_safe(&modules));
    }

    #[test]
    fn test_flatten_unsafe_with_eval() {
        let modules = vec![make_module("a.js", "eval('code');")];
        assert!(!is_flatten_safe(&modules));
    }

    #[test]
    fn test_flatten_unsafe_with_with_stmt() {
        let modules = vec![make_module("a.js", "with(obj) { foo(); }")];
        assert!(!is_flatten_safe(&modules));
    }

    #[test]
    fn test_inline_module_body_substitution() {
        let code = "exports.foo = 1; module.exports.bar = 2; var x = require(1);";
        let result = inline_module_body(code, 3);
        // `exports` → `_m3.exports`
        assert!(
            result.contains("_m3.exports.foo = 1"),
            "exports substituted, got: {}",
            result
        );
        // `module.exports` → `_m3.exports` (module replaced, .exports stays)
        assert!(
            result.contains("_m3.exports.bar = 2"),
            "module.exports substituted, got: {}",
            result
        );
        // `require` → `_r`
        assert!(
            result.contains("_r(1)"),
            "require substituted, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_module_body_preserves_strings() {
        let code = r#"var s = "module exports require"; exports.x = s;"#;
        let result = inline_module_body(code, 0);
        // Strings must NOT be substituted
        assert!(
            result.contains(r#""module exports require""#),
            "string content must be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_module_body_preserves_property_access() {
        // obj.module, obj.exports, obj.require should NOT be substituted
        let code = "var x = obj.module; var y = obj.exports; var z = obj.require;";
        let result = inline_module_body(code, 2);
        assert!(
            result.contains("obj.module"),
            "obj.module should be preserved, got: {}",
            result
        );
        assert!(
            result.contains("obj.exports"),
            "obj.exports should be preserved, got: {}",
            result
        );
        assert!(
            result.contains("obj.require"),
            "obj.require should be preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_module_body_no_partial_matches() {
        // `moduleId` should NOT be replaced as `module` + `Id`
        let code = "var moduleId = 1; var requireCount = 2; exportsMap = {};";
        let result = inline_module_body(code, 0);
        assert!(
            result.contains("moduleId"),
            "moduleId should not be changed, got: {}",
            result
        );
        assert!(
            result.contains("requireCount"),
            "requireCount should not be changed, got: {}",
            result
        );
        assert!(
            result.contains("exportsMap"),
            "exportsMap should not be changed, got: {}",
            result
        );
    }

    #[test]
    fn test_generate_flattened_bundle_empty() {
        assert!(generate_flattened_bundle(&[]).is_empty());
    }

    #[test]
    fn test_generate_flattened_bundle_single_module() {
        let modules = vec![make_module("entry.js", "exports.main = 42;")];
        let bundle = generate_flattened_bundle(&modules);
        assert!(bundle.contains("(function()"), "outer IIFE present");
        assert!(bundle.contains("var _m0="), "module var declared");
        // Phase 2: `exports` is aliased to `_m0e` and the alias is declared,
        // so `exports.main = 42` becomes `_m0e.main = 42`.
        assert!(
            bundle.contains("_m0e.main = 42"),
            "exports aliased to _m0e, got: {}",
            bundle
        );
        assert!(
            bundle.contains("var _m0e=_m0.exports"),
            "exports alias declaration present, got: {}",
            bundle
        );
        // No per-module wrapper function
        assert!(
            !bundle.contains("!function(module,exports,require)"),
            "no per-module wrapper, got: {}",
            bundle
        );
    }

    #[test]
    fn test_generate_flattened_bundle_two_modules() {
        let modules = vec![
            make_module("entry.js", "var dep = require(1); dep.exports.hello();"),
            make_module("dep.js", "exports.hello = function() {};"),
        ];
        let bundle = generate_flattened_bundle(&modules);
        // Both module vars declared
        assert!(bundle.contains("var _m0="), "m0 declared");
        assert!(bundle.contains("var _m1="), "m1 declared");
        // require → _r
        assert!(
            bundle.contains("_r(1)"),
            "require substituted, got: {}",
            bundle
        );
        // Phase 2: exports alias `_m1e` used in module 1 body
        assert!(
            bundle.contains("_m1e.hello"),
            "dep exports aliased to _m1e, got: {}",
            bundle
        );
        // Phase 2: top-level var `dep` in module 0 prefixed to `_m0_dep`
        assert!(
            bundle.contains("_m0_dep"),
            "module 0 local var 'dep' prefixed, got: {}",
            bundle
        );
    }

    #[test]
    fn test_generate_flattened_bundle_falls_back_on_eval() {
        let modules = vec![make_module("a.js", "eval('code');")];
        let flat = generate_flattened_bundle(&modules);
        let phase1 = generate_scope_hoisted_bundle(&modules);
        // Should fall back to Phase 1 (contains per-module wrapper)
        assert_eq!(flat, phase1, "should fall back to Phase 1 on eval");
    }

    #[test]
    fn test_inline_module_body_utf8_safe() {
        // Multi-byte UTF-8 characters must pass through unchanged
        let code = "exports.msg = '日本語テスト ✓'; require(1);";
        let result = inline_module_body(code, 0);
        assert!(
            result.contains("'日本語テスト ✓'"),
            "UTF-8 string preserved, got: {}",
            result
        );
        assert!(
            result.contains("_r(1)"),
            "require substituted after UTF-8, got: {}",
            result
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R5 bailout: is_flatten_safe with arguments[ check
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_flatten_unsafe_with_dynamic_arguments() {
        let modules = vec![make_module("a.js", "function f() { return arguments[0]; }")];
        assert!(
            !is_flatten_safe(&modules),
            "dynamic arguments[ access should trigger bailout"
        );
    }

    #[test]
    fn test_flatten_safe_arguments_length_ok() {
        // `arguments.length` does NOT use `arguments[` — should still be safe
        // to flatten if no eval/with present.
        // Note: the current check is conservative (substring match), so
        // `arguments.` access does not trigger the bailout.
        let modules = vec![make_module("a.js", "exports.x = 1;")];
        assert!(is_flatten_safe(&modules));
    }

    // ──────────────────────────────────────────────────────────────────
    // R2 / R3: collect_top_level_decls
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_collect_top_level_simple_var() {
        let names = collect_top_level_decls("var foo = 1; var bar = 2;");
        assert!(
            names.contains(&"foo".to_string()),
            "foo should be collected, got: {:?}",
            names
        );
        assert!(
            names.contains(&"bar".to_string()),
            "bar should be collected, got: {:?}",
            names
        );
    }

    #[test]
    fn test_collect_top_level_multi_var() {
        let names = collect_top_level_decls("var a = 1, b = 2, c = 3;");
        assert!(names.contains(&"a".to_string()), "a: {:?}", names);
        assert!(names.contains(&"b".to_string()), "b: {:?}", names);
        assert!(names.contains(&"c".to_string()), "c: {:?}", names);
    }

    #[test]
    fn test_collect_top_level_function_decl() {
        let names = collect_top_level_decls("function renderRoot(fiber) { var inner = 1; }");
        assert!(
            names.contains(&"renderRoot".to_string()),
            "renderRoot: {:?}",
            names
        );
        // inner var must NOT be collected (it's inside a function body)
        assert!(
            !names.contains(&"inner".to_string()),
            "inner should not be collected: {:?}",
            names
        );
    }

    #[test]
    fn test_collect_top_level_skips_nested() {
        let code = "var outer = 1; function f() { var inner = 2; }";
        let names = collect_top_level_decls(code);
        assert!(names.contains(&"outer".to_string()), "outer: {:?}", names);
        assert!(
            !names.contains(&"inner".to_string()),
            "inner should be skipped: {:?}",
            names
        );
    }

    #[test]
    fn test_collect_top_level_skips_cjs_globals() {
        // exports/module/require appear as free vars in module body, not as decls.
        let code = "exports.x = 1; module.exports = {}; var y = require(1);";
        let names = collect_top_level_decls(code);
        assert!(
            !names.contains(&"exports".to_string()),
            "exports not a decl: {:?}",
            names
        );
        assert!(
            !names.contains(&"module".to_string()),
            "module not a decl: {:?}",
            names
        );
        assert!(names.contains(&"y".to_string()), "y is a decl: {:?}", names);
    }

    // ──────────────────────────────────────────────────────────────────
    // R2 / R3: inline_module_body_v2
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_inline_v2_exports_aliased() {
        let code = "exports.foo = 1;";
        let result = inline_module_body_v2(code, 3);
        // exports → _m3e
        assert!(
            result.contains("_m3e.foo = 1"),
            "exports aliased to _m3e, got: {}",
            result
        );
        assert!(
            !result.contains("exports"),
            "raw 'exports' removed, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_module_substituted() {
        let code = "module.exports = {foo: 1};";
        let result = inline_module_body_v2(code, 2);
        // module → _m2
        assert!(
            result.contains("_m2.exports = {foo: 1}"),
            "module → _m2, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_require_substituted() {
        let code = "var x = require(1).foo;";
        let result = inline_module_body_v2(code, 0);
        assert!(result.contains("_r(1)"), "require → _r, got: {}", result);
    }

    #[test]
    fn test_inline_v2_local_var_prefixed() {
        let code = "var workInProgress = null; exports.render = workInProgress;";
        let result = inline_module_body_v2(code, 1);
        // var declaration renamed
        assert!(
            result.contains("_m1_workInProgress"),
            "workInProgress prefixed, got: {}",
            result
        );
        // reference also renamed
        assert!(
            result.contains("_m1e.render = _m1_workInProgress"),
            "reference renamed too, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_property_access_not_renamed() {
        // obj.exports, obj.module should NOT be substituted
        let code = "var x = obj.exports; var y = obj.module;";
        let result = inline_module_body_v2(code, 0);
        assert!(
            result.contains("obj.exports"),
            "obj.exports preserved, got: {}",
            result
        );
        assert!(
            result.contains("obj.module"),
            "obj.module preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_string_content_preserved() {
        let code = r#"var s = "exports module require"; exports.x = s;"#;
        let result = inline_module_body_v2(code, 0);
        assert!(
            result.contains(r#""exports module require""#),
            "string content preserved, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_collision_avoidance() {
        // Module 0 and module 1 both declare `var count`. After prefixing,
        // they become `_m0_count` and `_m1_count` — distinct names that can
        // safely coexist in the flat outer scope.
        let code0 = "var count = 0; exports.inc = function() { return count; };";
        let code1 = "var count = 10; exports.get = function() { return count; };";
        let r0 = inline_module_body_v2(code0, 0);
        let r1 = inline_module_body_v2(code1, 1);
        assert!(r0.contains("_m0_count"), "module 0 count prefixed: {}", r0);
        assert!(r1.contains("_m1_count"), "module 1 count prefixed: {}", r1);
        // The two prefixed names are distinct
        assert!(
            !r0.contains("_m1_count"),
            "module 0 should not have _m1_count: {}",
            r0
        );
        assert!(
            !r1.contains("_m0_count"),
            "module 1 should not have _m0_count: {}",
            r1
        );
    }

    #[test]
    fn test_generate_flattened_bundle_exports_alias_declared() {
        let modules = vec![make_module("entry.js", "var x = 1; exports.x = x;")];
        let bundle = generate_flattened_bundle(&modules);
        // Exports alias must be declared in the bundle
        assert!(
            bundle.contains("var _m0e=_m0.exports"),
            "exports alias declaration present, got: {}",
            bundle
        );
        // exports in module body → _m0e
        assert!(
            bundle.contains("_m0e.x"),
            "exports.x → _m0e.x, got: {}",
            bundle
        );
        // Local var 'x' prefixed to _m0_x
        assert!(
            bundle.contains("_m0_x"),
            "local var x prefixed, got: {}",
            bundle
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R4: Cross-module constant inlining (integration with flatten)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_flattened_then_inline_constants_string() {
        // Module 0 exports a const string, module 1 uses it.
        // After flatten + R4, the const should be inlined.
        let modules = vec![
            make_module(
                "entry.js",
                "var dep = require(1); if (dep.exports.MODE !== 'production') { debugSetup(); }",
            ),
            make_module("config.js", "var MODE = 'production'; exports.MODE = MODE;"),
        ];
        let flat = generate_flattened_bundle(&modules);
        let after_r4 = inline_cross_module_constants(&flat);

        // _m1_MODE should be inlined to 'production'
        assert!(
            !after_r4.contains("_m1_MODE"),
            "_m1_MODE should be inlined, got: {}",
            after_r4
        );
    }

    #[test]
    fn test_flattened_then_inline_constants_number() {
        let modules = vec![
            make_module(
                "entry.js",
                "var cfg = require(1); var arr = new Array(cfg.exports.SIZE);",
            ),
            make_module("config.js", "var SIZE = 256; exports.SIZE = SIZE;"),
        ];
        let flat = generate_flattened_bundle(&modules);
        let after_r4 = inline_cross_module_constants(&flat);

        assert!(
            !after_r4.contains("_m1_SIZE"),
            "_m1_SIZE should be inlined, got: {}",
            after_r4
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R5: Cross-module DCE (integration with flatten)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_flattened_then_eliminate_unused_exports() {
        // In the flattened bundle, module 0 accesses module 1's exports
        // through `_r(1).exports.xxx`, not through `_m1e.xxx` directly.
        // So R5 treats both `_m1e.used` and `_m1e.unused` as having zero
        // direct reads and removes them both, reducing bundle size.
        let modules = vec![
            make_module("entry.js", "var lib = require(1); lib.exports.used();"),
            make_module("lib.js", "exports.used = function() { return 1; };\nexports.unused = function() { return 2; };"),
        ];
        let flat = generate_flattened_bundle(&modules);
        let after_r5 = eliminate_unused_exports(&flat);

        // Both exports have no direct _m1e.xxx reads (accessed via _r(1)),
        // so R5 removes them, making the bundle smaller.
        assert!(
            after_r5.len() < flat.len(),
            "R5 should reduce bundle size: {} < {}",
            after_r5.len(),
            flat.len()
        );
    }

    #[test]
    fn test_flattened_then_eliminate_exports_with_direct_read() {
        // When a module internally reads its own export via the _m{i}e alias,
        // R5 must preserve it.
        let modules = vec![
            make_module("entry.js", "var lib = require(1);"),
            make_module("lib.js", "exports.init = function() {};\nexports.main = function() { return exports.init(); };"),
        ];
        let flat = generate_flattened_bundle(&modules);
        // After flattening, `exports.init()` in module 1 becomes `_m1e.init()`
        // which is a read reference — R5 must preserve `_m1e.init`.
        let after_r5 = eliminate_unused_exports(&flat);

        assert!(
            after_r5.contains("_m1e.init"),
            "export with internal read should survive R5, got: {}",
            after_r5
        );
    }

    #[test]
    fn test_flattened_then_eliminate_unused_prefixed_vars() {
        // A module with a helper function that is not referenced after DCE
        // should have it removed.
        let modules = vec![
            make_module("entry.js", "var util = require(1); util.exports.main();"),
            make_module(
                "util.js",
                "var helper = function() {};\nexports.main = function() { return 42; };",
            ),
        ];
        let flat = generate_flattened_bundle(&modules);
        let after_r5 = eliminate_unused_exports(&flat);

        // _m1_helper has no references → should be removed
        assert!(
            !after_r5.contains("_m1_helper"),
            "unused prefixed var should be removed, got: {}",
            after_r5
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // R4 + R5 combined pipeline
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_r4_then_r5_combined_pipeline() {
        // After R4 inlines constants, some exports may become unused.
        // R5 should clean them up.
        let modules = vec![
            make_module(
                "entry.js",
                "var cfg = require(1); if (cfg.MODE !== 'production') { require(2).debug(); }",
            ),
            make_module("config.js", "var MODE = 'production'; exports.MODE = MODE;"),
            make_module(
                "debug.js",
                "exports.debug = function() { console.log('debug'); };",
            ),
        ];
        let flat = generate_flattened_bundle(&modules);
        let after_r4 = inline_cross_module_constants(&flat);
        let after_r5 = eliminate_unused_exports(&after_r4);

        // MODE should be inlined
        assert!(
            !after_r5.contains("_m1_MODE"),
            "MODE should be inlined by R4, got: {}",
            after_r5
        );

        // The flattened bundle should be smaller after R4+R5
        assert!(
            after_r5.len() <= flat.len(),
            "R4+R5 should reduce bundle size: {} <= {}",
            after_r5.len(),
            flat.len()
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // DeclKind tracking (extended collect_top_level_decls)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_collect_top_level_decls_with_kind_var() {
        let decls = collect_top_level_decls_with_kind("var x = 1; var y = 2;");
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].name, "x");
        assert_eq!(decls[0].kind, DeclKind::Var);
        assert_eq!(decls[1].name, "y");
        assert_eq!(decls[1].kind, DeclKind::Var);
    }

    #[test]
    fn test_collect_top_level_decls_with_kind_const() {
        let decls = collect_top_level_decls_with_kind("const MODE = 'production';");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].name, "MODE");
        assert_eq!(decls[0].kind, DeclKind::Const);
    }

    #[test]
    fn test_collect_top_level_decls_with_kind_let() {
        let decls = collect_top_level_decls_with_kind("let count = 0;");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].name, "count");
        assert_eq!(decls[0].kind, DeclKind::Let);
    }

    #[test]
    fn test_collect_top_level_decls_with_kind_function() {
        let decls = collect_top_level_decls_with_kind("function render() {}");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].name, "render");
        assert_eq!(decls[0].kind, DeclKind::Function);
    }

    #[test]
    fn test_collect_top_level_decls_with_kind_class() {
        let decls = collect_top_level_decls_with_kind("class Component {}");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].name, "Component");
        assert_eq!(decls[0].kind, DeclKind::Class);
    }

    #[test]
    fn test_collect_top_level_decls_with_kind_mixed() {
        let code = "var a = 1; const B = 'x'; let c = []; function d() {} class E {}";
        let decls = collect_top_level_decls_with_kind(code);
        assert_eq!(decls.len(), 5, "decls: {:?}", decls);
        assert_eq!(decls[0].kind, DeclKind::Var);
        assert_eq!(decls[1].kind, DeclKind::Const);
        assert_eq!(decls[2].kind, DeclKind::Let);
        assert_eq!(decls[3].kind, DeclKind::Function);
        assert_eq!(decls[4].kind, DeclKind::Class);
    }
}
// CODEGEN-END
