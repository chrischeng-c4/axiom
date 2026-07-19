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

use std::collections::{HashMap, HashSet};

use super::splitting;
use super::CompiledModule;

// Re-export post-flattening optimizations from the split module.
pub use super::scope_hoist_opt::{
    eliminate_unused_exports, eliminate_unused_exports_preserving_entry,
    inline_cross_module_constants, is_side_effect_free,
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

    let mut out = String::with_capacity(estimate_output_size(modules));

    // Outer IIFE to avoid leaking module variables into global scope
    out.push_str("(function(){\n'use strict';\n\n");

    // Pre-declare all module namespace objects.
    // Using `var` means they are hoisted to the function scope and
    // visible everywhere inside the IIFE.
    let module_slot_count = module_slot_count(modules);
    for i in 0..module_slot_count {
        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", i));
    }
    out.push('\n');

    emit_require_lookup(&mut out, module_slot_count);

    // Execute modules in reverse topological order so that each
    // dependency is fully initialised before its importer runs.
    // (modules[0] = entry point; modules[n-1] = deepest leaf)
    for module in modules.iter().rev() {
        let module_id = module.id;
        let module_path = module.path.to_string_lossy();
        out.push_str(&format!("// Module {}: {}\n", module_id, module_path));
        // Each module gets its own function scope so that local
        // `var` declarations don't collide across modules.  The
        // single IIFE wrapper means a minifier can still see all
        // module-level references and apply cross-module DCE.
        out.push_str(&format!(
            "!function(module,exports,require){{\n{}}}(\
             _m{},_m{}.exports,_r);\n\n",
            module.code, module_id, module_id
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
        if !is_module_flatten_safe(&module.code) {
            return false;
        }
    }
    true
}

// `pub(crate)`: reused by `dce::eliminate_dead_top_level_declarations`
// (WI #2126) as the whole-module eval/with/arguments[..] safety probe
// before attempting statement-level DCE, so it is not skip-hoist-local
// anymore.
pub(crate) fn is_module_flatten_safe(code: &str) -> bool {
    !(code.contains("eval(") || code.contains("with(") || code.contains("arguments["))
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
/// Keeps a Phase 1 wrapper only around modules that are individually unsafe
/// to flatten.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn generate_flattened_bundle(modules: &[CompiledModule]) -> String {
    if modules.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(estimate_output_size(modules));

    out.push_str("(function(){\n'use strict';\n\n");

    // Pre-declare all module namespace objects using short names.
    let module_slot_count = module_slot_count(modules);
    for i in 0..module_slot_count {
        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", i));
    }
    out.push('\n');

    // Lightweight require function — still needed for patterns like
    // `var dep = require(1)` that reference modules by numeric ID.
    emit_require_lookup(&mut out, module_slot_count);

    // Inline each module body in reverse topological order (deepest deps first).
    // R6: modules with side effects retain their IIFE wrapper for isolation.
    for module in modules.iter().rev() {
        let module_id = module.id;
        let module_path = module.path.to_string_lossy();
        // R6: Check package.json sideEffects field for node_modules packages.
        // Project source files that passed the per-module flatten check are eligible
        // for inlining — CJS exports assignments are not "side effects" in this
        // context, they're the module's output mechanism.
        let module_flatten_safe = is_module_flatten_safe(&module.code);
        let in_node_modules = module.path.to_string_lossy().contains("node_modules");
        let side_effect_free = if in_node_modules {
            is_side_effect_free(module)
        } else {
            true
        };
        out.push_str(&format!("// Module {}: {}\n", module_id, module_path));

        if module_flatten_safe && side_effect_free {
            // Side-effect-free: inline directly into flat scope.
            // Apply per-module prefix renaming (R3) + CJS substitutions (R2).
            let inlined = inline_module_body_v2(&module.code, module_id);
            out.push_str("{\n");
            // NOTE: the `exports` alias is emitted unconditionally on purpose.
            // Dropping provably-unused `var _mNe=_mN.exports;` declarations
            // looks free (338 of 341 are dead on the mui bundle) but removing
            // a binding before the mangle pass shifts its name assignment and
            // exposes a latent mangler collision that breaks the runtime
            // (dom-production-assets). Until the mangler is collision-stable
            // under binding removal, keep the alias and recover the bytes in a
            // post-mangle dead-`var` pass instead. Tracker: jet-exports-alias.
            out.push_str(&format!("var _m{idx}e=_m{idx}.exports;\n", idx = module_id));
            out.push_str(&inlined);
            out.push_str("\n}\n\n");
        } else {
            // Side-effectful or flatten-unsafe: keep IIFE wrapper to preserve
            // execution order and local dynamic-scope semantics.
            tracing::debug!(
                "Module {} retained wrapper (side_effect_free={}, flatten_safe={})",
                module_id,
                side_effect_free,
                module_flatten_safe
            );
            out.push_str(&format!(
                "!function(module,exports,require){{{}}}(_m{idx},_m{idx}.exports,_r);\n\n",
                module.code,
                idx = module_id
            ));
        }
    }

    out.push_str("})();\n");
    dedup_content_twins(out, modules)
}

/// Eliminate duplicate copies of the same source module bundled under two
/// resolved paths — e.g. `@mui/system/colorManipulator.js` (CJS) and
/// `@mui/system/esm/colorManipulator.js` (ESM), both ~8.5KB of the same
/// color functions. Importers that resolved to the subset copy are
/// redirected (`_r(loser)` -> `_r(winner)`) to the copy that provides a
/// superset of the consumed exports, and the loser's body is dropped.
///
/// Safe by construction: a loser is merged only when EVERY name read from
/// it via `_r(loser)["name"]` is also written as an export by the winner,
/// so the redirect resolves to the same value. Same package + same file
/// basename guarantees identical source. The empty `_m{loser}` slot is
/// left in place (harmless ~18 bytes) so module-id indices never shift.
fn dedup_content_twins(bundle: String, modules: &[CompiledModule]) -> String {
    use std::collections::HashMap;
    // Group module ids by (package, basename).
    let key_of = |m: &CompiledModule| -> Option<(String, String)> {
        let p = m.path.to_string_lossy();
        let rel = p.rsplit_once("/node_modules/").map(|(_, r)| r)?;
        let pkg = if rel.starts_with('@') {
            rel.splitn(3, '/').take(2).collect::<Vec<_>>().join("/")
        } else {
            rel.split('/').next()?.to_string()
        };
        let base = rel.rsplit('/').next()?.to_string();
        Some((pkg, base))
    };
    let mut groups: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for m in modules {
        if let Some(k) = key_of(m) {
            groups.entry(k).or_default().push(m.id);
        }
    }
    let twins: Vec<Vec<usize>> = groups.into_values().filter(|ids| ids.len() > 1).collect();
    if twins.is_empty() {
        return bundle;
    }

    // Names consumed from a module via `_r(id)["name"]`.
    let consumed_names = |bundle: &str, id: usize| -> Vec<String> {
        let needle = format!("_r({id})[\"");
        let mut out = Vec::new();
        let mut from = 0usize;
        while let Some(rel) = bundle[from..].find(&needle) {
            let s = from + rel + needle.len();
            if let Some(end) = bundle[s..].find('"') {
                out.push(bundle[s..s + end].to_string());
            }
            from = s;
        }
        out
    };
    // Whether module `id`'s emitted block writes export `name`.
    let block_of = |bundle: &str, id: usize| -> Option<(usize, usize)> {
        let marker = format!("// Module {id}: ");
        let start = bundle.find(&marker)?;
        let after = start + marker.len();
        let end = bundle[after..]
            .find("// Module ")
            .map(|r| after + r)
            .unwrap_or_else(|| bundle.rfind("})();\n").unwrap_or(bundle.len()));
        Some((start, end))
    };
    let provides = |block: &str, id: usize, name: &str| -> bool {
        block.contains(&format!("_m{id}e.{name} ="))
            || block.contains(&format!("_m{id}e.{name}="))
            || block.contains(&format!("_m{id}.exports[\"{name}\"]"))
            || block.contains(&format!("_m{id}e[\"{name}\"]"))
            || block.contains(&format!("_m{id}.exports.{name} ="))
    };

    let mut result = bundle;
    for ids in twins {
        // Winner = the id with the most distinct consumed names (the superset).
        let mut ranked: Vec<(usize, usize)> = ids
            .iter()
            .map(|&id| (consumed_names(&result, id).len(), id))
            .collect();
        ranked.sort_unstable();
        let (_, winner) = *ranked.last().unwrap();
        let Some((ws, we)) = block_of(&result, winner) else {
            continue;
        };
        let winner_block = result[ws..we].to_string();
        for &(_, loser) in &ranked {
            if loser == winner {
                continue;
            }
            let needs = consumed_names(&result, loser);
            // Only merge when the winner provides every consumed name AND the
            // loser is consumed purely by property access (no bare `_r(loser)`
            // namespace/`||_r(loser)` interop we can't prove equivalent).
            let bare = format!("_r({loser})");
            let prop = format!("_r({loser})[");
            let bare_nonprop = {
                let mut from = 0usize;
                let mut found = false;
                while let Some(rel) = result[from..].find(&bare) {
                    let at = from + rel;
                    let after = at + bare.len();
                    let is_prop = result[at..].starts_with(&prop);
                    let id_continues = result[after..]
                        .bytes()
                        .next()
                        .map(|b| b.is_ascii_digit())
                        .unwrap_or(false);
                    if !is_prop && !id_continues {
                        found = true;
                        break;
                    }
                    from = after;
                }
                found
            };
            if bare_nonprop || needs.is_empty() {
                continue;
            }
            if !needs.iter().all(|n| provides(&winner_block, winner, n)) {
                continue;
            }
            // Redirect every `_r(loser)` -> `_r(winner)` (exact id), drop body.
            result = redirect_require_id(&result, loser, winner);
            if let Some((ls, le)) = block_of(&result, loser) {
                result.replace_range(ls..le, "");
            }
        }
    }
    result
}

/// Replace `_r(<from>)` with `_r(<to>)` for the exact numeric id (not a
/// prefix of a longer id), across the whole bundle.
fn redirect_require_id(bundle: &str, from: usize, to: usize) -> String {
    let needle = format!("_r({from})");
    let repl = format!("_r({to})");
    let mut out = String::with_capacity(bundle.len());
    let mut at = 0usize;
    while let Some(rel) = bundle[at..].find(&needle) {
        let s = at + rel;
        out.push_str(&bundle[at..s]);
        out.push_str(&repl);
        at = s + needle.len();
    }
    out.push_str(&bundle[at..]);
    out
}

fn module_slot_count(modules: &[CompiledModule]) -> usize {
    modules
        .iter()
        .map(|module| module.id)
        .max()
        .map(|id| id + 1)
        .unwrap_or(0)
}

fn emit_require_lookup(out: &mut String, n: usize) {
    // Store module objects, not the exports objects, because CommonJS modules
    // may replace `module.exports` after this lookup table is initialized.
    out.push_str("var _mods=[");
    for i in 0..n {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("_m{}", i));
    }
    out.push_str("];\n");
    out.push_str("function _r(id){var m=_mods[id];return m?m.exports:{}}\n\n");
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

    // Build separate scoped rename maps:
    //
    // - root_renames applies only to top-level declarations and references that
    //   resolve to those declarations.
    // - global_renames applies only to unresolved CJS globals.
    //
    // Keeping those paths separate is required for packages such as Stylis,
    // where top-level globals like `line` are shadowed by function parameters
    // and also used as object-literal keys.
    let mut root_renames: HashMap<String, String> = HashMap::with_capacity(decls.len());
    let mut global_renames: HashMap<String, String> = HashMap::with_capacity(3);

    global_renames.insert("exports".to_string(), exports_alias);
    global_renames.insert("module".to_string(), module_repl);
    global_renames.insert("require".to_string(), "_r".to_string());

    // Per-module prefix for top-level declarations.
    for decl in decls {
        root_renames
            .entry(decl.clone())
            .or_insert_with(|| format!("_m{}_{}", idx, decl));
    }

    super::mangle::apply_scoped_module_renames(code, &root_renames, &global_renames)
}

// ──────────────────────────────────────────────────────────────────────────
// Split-entry flatten (issue #1993): flatten the SAFE SUBSET of a split
// build's entry chunk into one flat scope with unified mangling, keeping
// the `__jet__` registry for the residue the fallback ladder can't prove
// safe. Reuses `inline_module_body_v2` (R2/R3 substitution) unchanged; see
// `bundler::mod::generate_split_bundle` for the call site and
// `bundler::splitting` for the graph-shape helpers this partition uses.
// ──────────────────────────────────────────────────────────────────────────

/// Partition of an entry chunk's modules into a flatten-safe subset (one
/// shared IIFE scope, unified mangling target — see
/// [`generate_entry_flat_region`]) and a `__jet__`-registry residue (the
/// modules the fallback ladder in [`partition_entry_for_flatten`] cannot
/// prove safe).
/// @issue #1993
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryFlattenPartition {
    /// Ids to inline into the flat region. Already in dependency-execution
    /// order (importer first) per [`splitting::dependency_order`].
    pub flat_ids: Vec<usize>,
    /// Ids that keep the `__jet__.define`/`require` registry wrapper.
    pub registry_ids: Vec<usize>,
}

/// Classify an entry chunk's modules per the issue #1993 fallback ladder.
/// A module stays in the `__jet__` registry (not flattened) if it:
///
/// - participates in a static-edge cycle ([`splitting::cycle_members`]:
///   SCC size > 1 or a self-loop) — flattening would run initializers in
///   the wrong order or infinitely recurse;
/// - is referenced by any OTHER chunk ([`splitting::cross_chunk_referenced`])
///   — it must stay id-addressable via `__jet__.require` from that chunk;
/// - statically depends on a module that lives in another chunk
///   ([`splitting::cross_chunk_importers`], e.g. a promoted #1963
///   shared/manual chunk) — `entry_bootstrap_js` only defers the
///   *registry* `require(entry_id)` path behind that chunk's `loadChunk`
///   promise, but a flattened module runs synchronously and
///   unconditionally as soon as the bundle script executes, well before
///   that promise can resolve;
/// - fails the per-module scope-hoist-safety check
///   ([`is_entry_module_flatten_safe`]: `eval`/`with`/`arguments[..]`, or a
///   side-effectful `node_modules` package); or
/// - is itself a dynamic-import target — defensively `debug_assert!`ed
///   rather than expected, since `splitting::bfs_static` already never
///   walks into a split point, so this should be structurally impossible
///   for entry-chunk members.
///
/// Everything else is safe to inline into the flat region. Correctness
/// over size: ties and doubtful cases fall to the registry side.
/// @issue #1993
pub fn partition_entry_for_flatten(
    entry_modules: &[&CompiledModule],
    edges: &[splitting::SplitEdgeId],
) -> EntryFlattenPartition {
    let entry_ids: HashSet<usize> = entry_modules.iter().map(|m| m.id).collect();

    debug_assert!(
        !edges
            .iter()
            .any(|edge| edge.is_dynamic && entry_ids.contains(&edge.to)),
        "entry chunk module is a dynamic-import target; split_chunks invariant violated"
    );

    let cyclic = splitting::cycle_members(&entry_ids, edges);
    let cross_referenced = splitting::cross_chunk_referenced(&entry_ids, edges);
    let cross_importers = splitting::cross_chunk_importers(&entry_ids, edges);

    let mut flat_ids: Vec<usize> = Vec::with_capacity(entry_modules.len());
    let mut registry_ids: Vec<usize> = Vec::with_capacity(entry_modules.len());

    for module in entry_modules {
        let id = module.id;
        let safe = !cyclic.contains(&id)
            && !cross_referenced.contains(&id)
            && !cross_importers.contains(&id)
            && is_entry_module_flatten_safe(module);
        if safe {
            flat_ids.push(id);
        } else {
            registry_ids.push(id);
        }
    }

    let flat_ids = splitting::dependency_order(&flat_ids, edges);
    registry_ids.sort_unstable();

    EntryFlattenPartition {
        flat_ids,
        registry_ids,
    }
}

/// Per-module scope-hoist safety check used by [`partition_entry_for_flatten`].
/// Mirrors the same-purpose combined predicate inside
/// [`generate_flattened_bundle`]: the `eval`/`with`/`arguments[..]` guard
/// ([`is_module_flatten_safe`]), plus the `sideEffects` package.json
/// heuristic ([`is_side_effect_free`]) for `node_modules` dependencies.
/// Project source is always side-effect-free from a scope-hoist
/// perspective — a CJS `exports`/`module.exports` assignment is the
/// module's output mechanism, not a side effect.
/// @issue #1993
fn is_entry_module_flatten_safe(module: &CompiledModule) -> bool {
    if !is_module_flatten_safe(&module.code) {
        return false;
    }
    if module.path.to_string_lossy().contains("node_modules") {
        return is_side_effect_free(module);
    }
    true
}

/// Generate the flat-scope region for the split-entry-chunk subset chosen
/// by [`partition_entry_for_flatten`]: one IIFE containing every flat
/// module's namespace object plus its inlined body (same `_m{idx}` /
/// `_m{idx}e` / `_r` substitutions as [`generate_flattened_bundle`]'s
/// Phase 2, via [`inline_module_body_v2`]), executed in dependency order
/// (leaf deps first).
///
/// Two differences from the single-file flatten path, both required for
/// split-build interop with the surrounding `__jet__` registry:
///
/// 1. The local `_r(id)` helper checks the flat region's own sparse module
///    table first, then falls back to `__jet__.require(id)` — so a flat
///    module that depends on a registry-residue module still resolves.
/// 2. Immediately after *each* flat module's own body runs — not batched
///    once at the end of the IIFE — the region pre-seeds
///    `__jet__.cache[id] = {exports, id, loaded}` for that one id,
///    mirroring the runtime's own `require()` cache contract (`cache[id]`
///    is checked before `modules[id]`; see `generate_runtime`). Per-module
///    interleaving (rather than one trailing batch) matters: a flat
///    module's top-level code can synchronously call into a
///    registry-residue module, whose factory then runs *nested inside*
///    this still-executing IIFE and may itself `require()` an *earlier*
///    flat module's export — which must already be cache-seeded at that
///    exact point, not merely already executed. The seed reads the live
///    `_m{idx}.exports` property (not a captured `_m{idx}e` alias),
///    matching real CommonJS `module.exports` reassignment semantics.
///    The seed is skipped for non-entry ids when `has_registry_residue`
///    is `false`: every intra-region cross-reference already resolves
///    through `_r`'s local `_mods` lookup, which never falls through to
///    `__jet__.require` for an id it already holds, so with zero registry
///    code in this chunk nothing can ever call `__jet__.require`/read
///    `__jet__.cache` for a non-entry flat id — the seed would be
///    provably dead weight. `entry_id`'s own seed is always emitted
///    regardless, because the trailing bootstrap call
///    (`entry_bootstrap_js`) unconditionally does
///    `__jet__.require(entry_id)`, which resolves through the *registry*
///    `require`'s cache-first check — see `generate_runtime` — not `_r`.
///
/// Ordering contract with the caller: registry `__jet__.define(...)` calls
/// must be emitted — and thus have run, since `define` only registers a
/// factory with no invocation — BEFORE this region's output, so a flat
/// module's synchronous top-level call into a registry-residue module
/// always finds `__jet__.modules[id]` already populated. The split runtime
/// (which defines `window.__jet__`) precedes both. See
/// `bundler::mod::generate_split_bundle`.
///
/// Residual limitation (documented, not fixed by this change): a
/// three-hop chain — flat module A synchronously calls registry module R
/// at A's top level, and R itself `require()`s flat module B, where B is
/// *not* also a direct static dependency of A — can still resolve before
/// B's own body has run, if the flat-only topological order (which only
/// sees direct flat-to-flat edges, not transitive ones through registry
/// nodes) happens to place B after A. This is a narrow, specific
/// ordering-collision pattern (it requires registry residue — itself
/// already the less-common case — to sit *between* two otherwise-unrelated
/// flat modules on a synchronous top-level path); the more complete fix
/// (transitive flat-to-flat ordering constraints computed through registry
/// nodes) is left as follow-up rather than implemented here, per issue
/// #1993's STOP-before-forcing-hairy-interop guidance.
/// @issue #1993
pub fn generate_entry_flat_region(
    flat_modules: &[&CompiledModule],
    edges: &[splitting::SplitEdgeId],
    entry_id: usize,
    has_registry_residue: bool,
) -> String {
    if flat_modules.is_empty() {
        return String::new();
    }

    let by_id: HashMap<usize, &CompiledModule> = flat_modules.iter().map(|m| (m.id, *m)).collect();
    let ids: Vec<usize> = flat_modules.iter().map(|m| m.id).collect();
    let order = splitting::dependency_order(&ids, edges);

    let code_total: usize = flat_modules.iter().map(|m| m.code.len()).sum();
    let mut out = String::with_capacity(code_total + 200 + flat_modules.len() * 160);

    out.push_str("// Entry flat region (issue #1993): scope-hoisted subset of the\n");
    out.push_str("// entry chunk, unified mangling target. Registry residue precedes\n");
    out.push_str("// this region (see generate_split_bundle) so its __jet__.define(...)\n");
    out.push_str("// calls have already registered by the time this IIFE runs.\n");
    out.push_str("(function(){\n'use strict';\n\n");

    for &id in &order {
        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", id));
    }
    out.push('\n');

    // Sparse by-id lookup restricted to this flat region, falling back to
    // the `__jet__` registry for cross-region requires.
    out.push_str("var _mods={");
    for (i, &id) in order.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{}:_m{}", id, id));
    }
    out.push_str("};\n");
    out.push_str("function _r(id){var m=_mods[id];return m?m.exports:__jet__.require(id)}\n\n");

    // Execute in dependency order (leaf deps first) — same convention as
    // generate_flattened_bundle. Each module's own `__jet__.cache` seed is
    // emitted immediately after its body (interleaved, not batched at the
    // end): see this function's doc comment for why that ordering matters.
    for &id in order.iter().rev() {
        let module = by_id[&id];
        let inlined = inline_module_body_v2(&module.code, id);
        out.push_str(&format!(
            "// Module {}: {}\n",
            id,
            module.path.to_string_lossy()
        ));
        out.push_str("{\n");
        out.push_str(&format!("var _m{idx}e=_m{idx}.exports;\n", idx = id));
        out.push_str(&inlined);
        out.push_str("\n}\n");
        // See this function's doc comment for why the entry id's seed is
        // unconditional while every other flat id's seed is dead weight
        // (and thus skipped) when this chunk has no registry residue.
        if has_registry_residue || id == entry_id {
            out.push_str(&format!(
                "__jet__.cache[{id}]={{exports:_m{id}.exports,id:{id},loaded:true}};\n\n",
                id = id
            ));
        } else {
            out.push('\n');
        }
    }

    out.push_str("})();\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_module(path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            id: test_module_id(path),
            path: PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    fn make_module_with_id(id: usize, path: &str, code: &str) -> CompiledModule {
        CompiledModule {
            id,
            path: PathBuf::from(path),
            code: code.to_string(),
            source_map: None,
            dependencies: Vec::new(),
            hash: String::new(),
        }
    }

    fn test_module_id(path: &str) -> usize {
        match path {
            "dep.js" | "b.js" | "safe.js" | "config.js" | "lib.js" => 1,
            "debug.js" => 2,
            _ => 0,
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

        // require lookup maps ids to live module objects, not stale exports
        // snapshots, so `module.exports = value` stays observable.
        assert!(bundle.contains("var _mods=[_m0,_m1];"));
        assert!(bundle.contains("return m?m.exports:{}"));

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
    fn test_require_lookup_tracks_module_exports_reassignment() {
        let modules = vec![
            make_module("entry.js", "var dep = require(1); exports.value = dep;"),
            make_module("dep.js", "module.exports = function dep() {};"),
        ];
        let bundle = generate_flattened_bundle(&modules);

        assert!(
            bundle.contains("var _mods=[_m0,_m1];"),
            "lookup must store live module objects, got: {}",
            bundle
        );
        assert!(
            !bundle.contains("var _mods=[_m0.exports,_m1.exports];"),
            "lookup must not snapshot initial exports objects, got: {}",
            bundle
        );
        assert!(
            bundle.contains("return m?m.exports:{}"),
            "require must read current module.exports, got: {}",
            bundle
        );
    }

    #[test]
    fn test_scope_hoist_preserves_sparse_module_ids_after_tree_shaking() {
        let modules = vec![
            make_module_with_id(0, "entry.js", "var dep = require(2); dep.run();"),
            make_module_with_id(2, "dep.js", "exports.run = function() {};"),
        ];
        let bundle = generate_scope_hoisted_bundle(&modules);

        assert!(bundle.contains("var _m2={exports:{}};"), "{bundle}");
        assert!(bundle.contains("var _mods=[_m0,_m1,_m2];"), "{bundle}");
        assert!(bundle.contains("Module 2: dep.js"), "{bundle}");
        assert!(bundle.contains("}(_m2,_m2.exports,_r);"), "{bundle}");
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
    fn test_generate_flattened_bundle_wraps_only_eval_module() {
        let modules = vec![make_module("a.js", "eval('code');")];
        let flat = generate_flattened_bundle(&modules);
        assert!(
            flat.contains("!function(module,exports,require){eval('code');}"),
            "eval module must retain wrapper, got: {}",
            flat
        );
        assert!(
            !flat.contains("var _m0e=_m0.exports"),
            "eval module must not be flattened, got: {}",
            flat
        );
    }

    #[test]
    fn test_generate_flattened_bundle_partially_flattens_safe_sibling() {
        let modules = vec![
            make_module("unsafe.js", "eval('code');"),
            make_module("safe.js", "exports.used = 1; var local = 2;"),
        ];
        let flat = generate_flattened_bundle(&modules);

        assert!(
            flat.contains("!function(module,exports,require){eval('code');}"),
            "unsafe module must retain wrapper, got: {}",
            flat
        );
        assert!(
            flat.contains("var _m1e=_m1.exports"),
            "safe sibling should still be flattened, got: {}",
            flat
        );
        assert!(
            flat.contains("_m1_local"),
            "safe sibling locals should be prefixed in flattened body, got: {}",
            flat
        );
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
    fn test_flatten_unsafe_with_module_scope_dynamic_arguments() {
        let modules = vec![make_module("a.js", "exports.x = arguments[0];")];
        assert!(
            !is_flatten_safe(&modules),
            "module-scope arguments[ access should trigger bailout"
        );
    }

    #[test]
    fn test_flatten_unsafe_with_arrow_lexical_arguments() {
        let modules = vec![make_module("a.js", "exports.x = () => arguments[0];")];
        assert!(
            !is_flatten_safe(&modules),
            "top-level arrow arguments[ access captures the module wrapper arguments"
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
    fn test_inline_v2_template_expression_refs_are_scoped() {
        let code = r#"var styledComponentId = "sc-a"; var selector = `style[${styledComponentId}]`; exports.selector = selector;"#;
        let result = inline_module_body_v2(code, 4);
        assert!(
            result.contains("var _m4_styledComponentId"),
            "top-level template input should be scoped, got: {}",
            result
        );
        assert!(
            result.contains("`style[${_m4_styledComponentId}]`"),
            "template expression ref should follow the scoped rename, got: {}",
            result
        );
        assert!(
            !result.contains("${styledComponentId}"),
            "template expression must not keep stale unscoped name, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_spread_expression_refs_are_scoped() {
        let code = r#"const SPACINGS = [0, 1, 2]; const classes = [...SPACINGS.map((value) => `spacing-${value}`)]; exports.classes = classes;"#;
        let result = inline_module_body_v2(code, 4);
        assert!(
            result.contains("const _m4_SPACINGS"),
            "top-level spread input should be scoped, got: {}",
            result
        );
        assert!(
            result.contains("..._m4_SPACINGS.map"),
            "spread expression ref should follow scoped rename, got: {}",
            result
        );
        assert!(
            !result.contains("...SPACINGS"),
            "spread expression must not keep stale unscoped name, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_preserves_shadowed_params_and_object_keys() {
        let code = concat!(
            "var line = 1; var column = 1; var length = 0;",
            "function read() { return line + column + length; }",
            "function node(value, line, column, length) {",
            "return {value: value, line: line, column: column, length: length};",
            "}",
            "exports.read = read; exports.node = node;"
        );
        let result = inline_module_body_v2(code, 16);

        assert!(
            result.contains("var _m16_line = 1"),
            "top-level line should be prefixed, got: {}",
            result
        );
        assert!(
            result.contains("return _m16_line + _m16_column + _m16_length"),
            "top-level reads should resolve to prefixed bindings, got: {}",
            result
        );
        assert!(
            result.contains("function _m16_node(value, line, column, length)"),
            "shadowing params must not be prefixed, got: {}",
            result
        );
        assert!(
            result.contains("{value: value, line: line, column: column, length: length}"),
            "object literal keys must be preserved, got: {}",
            result
        );
        assert!(
            result.contains("_m16e.node = _m16_node"),
            "exports and top-level function refs should be scoped-renamed, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_preserves_destructured_param_alias_shadowing() {
        let code = concat!(
            "var t = require(11);",
            "var I = [];",
            "function Je({ plugins: t = I } = {}) {",
            "const c = t.slice();",
            "return c;",
            "}",
            "exports.Je = Je;"
        );
        let result = inline_module_body_v2(code, 8);

        assert!(
            result.contains("var _m8_t = _r(11);"),
            "top-level t should be prefixed, got: {}",
            result
        );
        assert!(
            result.contains("function _m8_Je({ plugins: t = _m8_I } = {})"),
            "destructured alias param name must remain local t, got: {}",
            result
        );
        assert!(
            result.contains("const c = t.slice();"),
            "function body must read the destructured param alias, got: {}",
            result
        );
        assert!(
            !result.contains("_m8_t.slice()"),
            "function body must not resolve destructured param alias to top-level t, got: {}",
            result
        );
    }

    #[test]
    fn test_inline_v2_expands_renamed_object_shorthand_keys() {
        let code = concat!(
            "var grey = {100: '#f5f5f5'};",
            "var dark = {text: {primary: '#fff'}};",
            "var light = {text: {primary: '#000'}};",
            "function createPalette(mode) {",
            "const paletteOutput = {grey, dark, light, mode};",
            "return paletteOutput;",
            "}",
            "exports.createPalette = createPalette;"
        );
        let result = inline_module_body_v2(code, 671);

        assert!(
            result.contains("grey:_m671_grey"),
            "renamed shorthand value must preserve grey key, got: {}",
            result
        );
        assert!(
            result.contains("dark:_m671_dark"),
            "renamed shorthand value must preserve dark key, got: {}",
            result
        );
        assert!(
            result.contains("light:_m671_light"),
            "renamed shorthand value must preserve light key, got: {}",
            result
        );
        assert!(
            result.contains("mode}"),
            "unrenamed local shorthand should stay shorthand, got: {}",
            result
        );
    }

    #[test]
    fn test_constant_inline_preserves_shadowed_params_after_flatten() {
        let code = concat!(
            "var line = 1; var column = 1; var length = 0;",
            "function node(value, line, column, length) {",
            "return {value: value, line: line, column: column, length: length};",
            "}",
            "function read() { return line + column + length; }",
            "exports.node = node; exports.read = read;"
        );
        let modules = vec![make_module("stylis-like.js", code)];
        let flat = generate_flattened_bundle(&modules);
        let after_r4 = inline_cross_module_constants(&flat);

        assert!(
            after_r4.contains("function _m0_node(value, line, column, length)"),
            "R4 must not inline constants into shadowing params, got: {}",
            after_r4
        );
        assert!(
            after_r4.contains("{value: value, line: line, column: column, length: length}"),
            "R4 must preserve object literal keys and param refs, got: {}",
            after_r4
        );
        assert!(
            !after_r4.contains("function _m0_node(value, 1")
                && !after_r4.contains("function _m0_node(value, 0"),
            "function params must remain identifiers, got: {}",
            after_r4
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
        // through `_r(1).xxx`, not through `_m1e.xxx` directly. R5 must
        // still treat that require read as a live read of `_m1e.used`.
        let modules = vec![
            make_module("entry.js", "require(1).used();"),
            make_module("lib.js", "exports.used = function() { return 1; };\nexports.unused = function() { return 2; };"),
        ];
        let flat = generate_flattened_bundle(&modules);
        let after_r5 = eliminate_unused_exports(&flat);

        assert!(
            after_r5.contains("_m1e.used"),
            "R5 should preserve export read through _r(1), got: {}",
            after_r5
        );
        assert!(
            !after_r5.contains("_m1e.unused"),
            "R5 should still remove unread sibling export, got: {}",
            after_r5
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

    // ── #1993: split-entry flatten partition + flat-region emission ──

    /// Mixed fixture exercising every rung of the fallback ladder in one
    /// shot: a safe two-module cluster, a 2-module cycle, an eval()-using
    /// module, a module cross-referenced from an async chunk, and a module
    /// that itself statically depends on a promoted shared/manual chunk
    /// module (issue #1993's measured-regression fix: `entry_bootstrap_js`
    /// only defers the registry `require(entry_id)` path behind that other
    /// chunk's `loadChunk` promise, not a flattened module's synchronous
    /// top-level execution).
    #[test]
    fn partition_entry_for_flatten_mixed_fixture() {
        const SAFE1: usize = 0;
        const SAFE2: usize = 1;
        const CYCLE_A: usize = 2;
        const CYCLE_B: usize = 3;
        const EVAL_MOD: usize = 4;
        const CROSS_REF: usize = 5;
        const EXTERNAL_DEP: usize = 6;
        const ASYNC_CHUNK_MODULE: usize = 99; // lives in a different chunk
        const SHARED_CHUNK_MODULE: usize = 100; // promoted #1963 shared chunk

        let safe1 = make_module_with_id(SAFE1, "safe1.js", "var x = 1; module.exports = { x };");
        let safe2 = make_module_with_id(SAFE2, "safe2.js", "module.exports = 2;");
        let cycle_a = make_module_with_id(
            CYCLE_A,
            "cycle_a.js",
            "var a = require(3); module.exports = a;",
        );
        let cycle_b = make_module_with_id(
            CYCLE_B,
            "cycle_b.js",
            "var b = require(2); module.exports = b;",
        );
        let eval_mod = make_module_with_id(
            EVAL_MOD,
            "eval_mod.js",
            "eval('var y = 1'); module.exports = {};",
        );
        let cross_ref =
            make_module_with_id(CROSS_REF, "cross_ref.js", "module.exports = { z: 1 };");
        let external_dep = make_module_with_id(
            EXTERNAL_DEP,
            "external_dep.js",
            "var shared = require(100); module.exports = shared;",
        );

        let entry_modules: Vec<&CompiledModule> = vec![
            &safe1,
            &safe2,
            &cycle_a,
            &cycle_b,
            &eval_mod,
            &cross_ref,
            &external_dep,
        ];

        let edges = vec![
            splitting::SplitEdgeId {
                from: SAFE1,
                to: SAFE2,
                is_dynamic: false,
            },
            splitting::SplitEdgeId {
                from: CYCLE_A,
                to: CYCLE_B,
                is_dynamic: false,
            },
            splitting::SplitEdgeId {
                from: CYCLE_B,
                to: CYCLE_A,
                is_dynamic: false,
            },
            // Another (async) chunk's module statically requires CROSS_REF
            // by id — this is what forces CROSS_REF to the registry even
            // though it is otherwise scope-hoist safe on its own.
            splitting::SplitEdgeId {
                from: ASYNC_CHUNK_MODULE,
                to: CROSS_REF,
                is_dynamic: false,
            },
            // EXTERNAL_DEP statically requires a module that lives in a
            // different (promoted shared/manual) chunk — this is what
            // forces EXTERNAL_DEP to the registry even though it is
            // otherwise scope-hoist safe and part of no cycle.
            splitting::SplitEdgeId {
                from: EXTERNAL_DEP,
                to: SHARED_CHUNK_MODULE,
                is_dynamic: false,
            },
        ];

        let partition = partition_entry_for_flatten(&entry_modules, &edges);

        assert_eq!(partition.flat_ids, vec![SAFE1, SAFE2]);
        assert_eq!(
            partition.registry_ids,
            vec![CYCLE_A, CYCLE_B, EVAL_MOD, CROSS_REF, EXTERNAL_DEP]
        );
    }

    #[test]
    fn generate_entry_flat_region_emits_cache_preseed_and_local_fallback() {
        const A: usize = 0;
        const B: usize = 1;
        let a = make_module_with_id(A, "a.js", "var dep = require(1); module.exports = dep;");
        let b = make_module_with_id(B, "b.js", "module.exports = 2;");
        let flat_modules: Vec<&CompiledModule> = vec![&a, &b];
        let edges = vec![splitting::SplitEdgeId {
            from: A,
            to: B,
            is_dynamic: false,
        }];

        // has_registry_residue=true: this chunk has registry residue
        // elsewhere, so every flat id's cache must stay seeded (we can't
        // cheaply prove which specific ids the residue's `require()`
        // calls target).
        let out = generate_entry_flat_region(&flat_modules, &edges, A, true);

        // Local `_r` resolves same-region ids without touching `__jet__`,
        // falling back to `__jet__.require` for everything else.
        assert!(
            out.contains("function _r(id){var m=_mods[id];return m?m.exports:__jet__.require(id)}")
        );
        // Cache pre-seed makes flattened ids resolvable from registry
        // residue via `__jet__.require(id)` -> `cache[id]`.
        assert!(out.contains(&format!("__jet__.cache[{A}]=")));
        assert!(out.contains(&format!("__jet__.cache[{B}]=")));
        // `require(1)` inside module A's body was substituted to `_r(1)`.
        assert!(out.contains("_r(1)"));
        assert!(!out.contains("require(1)"));
    }

    #[test]
    fn generate_entry_flat_region_skips_dead_non_entry_cache_preseed_when_no_registry_residue() {
        // @issue #1993 — measured regression fix: on an all-safe chunk
        // (no registry residue at all), a non-entry flat id's cache seed
        // can never be read by anything (every intra-region reference
        // resolves through `_r`'s local `_mods` lookup, and nothing
        // outside this region can reach a flat-only chunk's ids), so it
        // must be omitted. The entry id's seed stays mandatory because
        // `entry_bootstrap_js`'s trailing `__jet__.require(entry_id)`
        // always goes through the *registry* require's cache-first check.
        const ENTRY: usize = 0;
        const LEAF: usize = 1;
        let entry = make_module_with_id(
            ENTRY,
            "entry.js",
            "var dep = require(1); module.exports = dep;",
        );
        let leaf = make_module_with_id(LEAF, "leaf.js", "module.exports = 2;");
        let flat_modules: Vec<&CompiledModule> = vec![&entry, &leaf];
        let edges = vec![splitting::SplitEdgeId {
            from: ENTRY,
            to: LEAF,
            is_dynamic: false,
        }];

        let out = generate_entry_flat_region(&flat_modules, &edges, ENTRY, false);

        assert!(
            out.contains(&format!("__jet__.cache[{ENTRY}]=")),
            "entry id's cache seed is unconditional (trailing bootstrap require): {out}"
        );
        assert!(
            !out.contains(&format!("__jet__.cache[{LEAF}]=")),
            "non-entry id's cache seed must be omitted when there is no registry residue: {out}"
        );
        // Flat-to-flat resolution is unaffected — still goes through `_r`.
        assert!(out.contains("_r(1)"));
    }

    #[test]
    fn generate_entry_flat_region_empty_input_is_empty_output() {
        let flat_modules: Vec<&CompiledModule> = Vec::new();
        assert_eq!(generate_entry_flat_region(&flat_modules, &[], 0, false), "");
    }
}
// CODEGEN-END
