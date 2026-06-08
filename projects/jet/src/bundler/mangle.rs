// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Variable mangling: scope analysis + variable renaming.
//!
//! Conservative approach: only renames function-local variables and parameters.
//! Preserves globals, property names, and module-level declarations.

use std::collections::{HashMap, HashSet};

/// Mangle local variable names to short identifiers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn mangle_variables(source: &str) -> String {
    mangle_variables_inner(source, false)
}

/// Mangle variables, optionally treating root scope as a function scope.
///
/// When `mangle_root_scope` is true, scope 0 declarations are mangled
/// just like any other function scope. This is correct when the source
/// is wrapped in an IIFE (e.g., scope-hoisted bundles).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn mangle_variables_with_root(source: &str) -> String {
    mangle_variables_inner(source, true)
}

fn mangle_variables_inner(source: &str, mangle_root: bool) -> String {
    let tokens = tokenize(source);
    if tokens.is_empty() {
        return source.to_string();
    }
    let si = build_scopes(source, &tokens);
    let renames = compute_renames(source, &tokens, &si, mangle_root);
    apply_renames(source, &tokens, &si.token_scope, &si.scopes, &renames)
}

// === Tokenizer ===

#[derive(Debug, Clone, Copy, PartialEq)]
enum TK {
    Ident,
    Str,
    Num,
    Regex,
    Punct,
}

#[derive(Debug, Clone)]
struct Tok {
    kind: TK,
    start: usize,
    end: usize,
}

fn txt<'a>(s: &'a str, t: &Tok) -> &'a str {
    &s[t.start..t.end]
}

fn tokenize(source: &str) -> Vec<Tok> {
    let b = source.as_bytes();
    let len = b.len();
    let mut toks = Vec::new();
    let mut i = 0;
    let mut prev = TK::Punct;

    while i < len {
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if is_id_start(b[i]) {
            let s = i;
            while i < len && is_id_cont(b[i]) {
                i += 1;
            }
            toks.push(Tok {
                kind: TK::Ident,
                start: s,
                end: i,
            });
            prev = TK::Ident;
            continue;
        }
        if b[i].is_ascii_digit() || (b[i] == b'.' && i + 1 < len && b[i + 1].is_ascii_digit()) {
            let s = i;
            if b[i] == b'0'
                && i + 1 < len
                && matches!(b[i + 1], b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
            {
                i += 2;
            } else {
                i += 1;
            }
            while i < len && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            if i < len && matches!(b[i], b'e' | b'E') {
                i += 1;
                if i < len && matches!(b[i], b'+' | b'-') {
                    i += 1;
                }
                while i < len && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            toks.push(Tok {
                kind: TK::Num,
                start: s,
                end: i,
            });
            prev = TK::Num;
            continue;
        }
        if matches!(b[i], b'"' | b'\'' | b'`') {
            let s = i;
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
            toks.push(Tok {
                kind: TK::Str,
                start: s,
                end: i,
            });
            prev = TK::Str;
            continue;
        }
        if b[i] == b'/' {
            if i + 1 < len && b[i + 1] == b'/' {
                while i < len && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if i + 1 < len && b[i + 1] == b'*' {
                i += 2;
                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            let is_rx = match prev {
                TK::Ident | TK::Num | TK::Str => false,
                _ => {
                    if let Some(last) = toks.last() {
                        let lt = txt(source, last);
                        lt != ")" && lt != "]"
                    } else {
                        true
                    }
                }
            };
            if is_rx && i + 1 < len && b[i + 1] != b'/' && b[i + 1] != b'*' {
                let s = i;
                i += 1;
                while i < len && b[i] != b'/' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    if i < len && b[i] == b'[' {
                        i += 1;
                        while i < len && b[i] != b']' {
                            if b[i] == b'\\' {
                                i += 1;
                            }
                            i += 1;
                        }
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                while i < len && b[i].is_ascii_alphabetic() {
                    i += 1;
                }
                toks.push(Tok {
                    kind: TK::Regex,
                    start: s,
                    end: i,
                });
                prev = TK::Regex;
                continue;
            }
        }
        let s = i;
        i += 1;
        toks.push(Tok {
            kind: TK::Punct,
            start: s,
            end: i,
        });
        prev = TK::Punct;
    }
    toks
}

fn is_id_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}
fn is_id_cont(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

// === Scope Analysis ===

struct Scope {
    parent: Option<usize>,
    is_function: bool,
    decls: HashSet<String>,
}

struct ScopeInfo {
    scopes: Vec<Scope>,
    token_scope: Vec<usize>,
}

fn build_scopes(source: &str, tokens: &[Tok]) -> ScopeInfo {
    let n = tokens.len();
    let mut scopes = vec![Scope {
        parent: None,
        is_function: true,
        decls: HashSet::new(),
    }];
    let mut stack: Vec<usize> = vec![0];
    let mut ts = vec![0usize; n];
    let mut pending_fn: Option<usize> = None;
    let mut in_decl = false; // between var/let/const and ;
    let mut expect_decl_name = false; // expecting the next ident to be a decl name
    let mut decl_is_var = false; // true if current decl is `var` (function-scoped)
    let mut decl_paren_depth: i32 = 0; // track parens inside declarations
    let mut decl_brace_depth: i32 = 0; // track braces inside declarations (object literals)
                                       // Save/restore decl state when entering/leaving function scopes.
                                       // Without this, `var o = {f: function() { var inner = 1; }}, next = 0;`
                                       // would lose the outer `in_decl` when the inner `var` resets it.
    let mut decl_state_stack: Vec<(bool, bool, bool, i32, i32)> = Vec::new();
    let mut expect_fn_params = false;
    let mut in_params = false;
    let mut param_depth = 0;
    let mut param_scope: Option<usize> = None;

    for i in 0..n {
        let cur = *stack.last().unwrap();
        ts[i] = cur;
        let tok = &tokens[i];

        if tok.kind == TK::Ident {
            let name = txt(source, tok);
            // Skip ident after dot (property access)
            if i > 0 && tokens[i - 1].kind == TK::Punct && txt(source, &tokens[i - 1]) == "." {
                expect_fn_params = false;
                continue;
            }
            match name {
                "function" => {
                    expect_fn_params = true;
                    continue;
                }
                "var" | "let" | "const" => {
                    in_decl = true;
                    expect_decl_name = true;
                    decl_is_var = name == "var";
                    decl_paren_depth = 0;
                    decl_brace_depth = 0;
                    continue;
                }
                _ => {}
            }
            if expect_fn_params {
                scopes[cur].decls.insert(name.to_string());
                continue;
            }
            if in_params {
                if let Some(ps) = param_scope {
                    scopes[ps].decls.insert(name.to_string());
                    ts[i] = ps;
                }
                continue;
            }
            if expect_decl_name {
                if decl_is_var {
                    // `var` is function-scoped: hoist to nearest function scope
                    let target = stack
                        .iter()
                        .rev()
                        .find(|&&s| scopes[s].is_function)
                        .copied()
                        .unwrap_or(cur);
                    scopes[target].decls.insert(name.to_string());
                } else {
                    // `let`/`const` are block-scoped
                    scopes[cur].decls.insert(name.to_string());
                }
                expect_decl_name = false;
                continue;
            }
        }

        if tok.kind == TK::Punct {
            let p = txt(source, tok);
            match p {
                "(" => {
                    if expect_fn_params {
                        let new_id = scopes.len();
                        scopes.push(Scope {
                            parent: Some(cur),
                            is_function: true,
                            decls: HashSet::new(),
                        });
                        in_params = true;
                        param_depth = 1;
                        param_scope = Some(new_id);
                        pending_fn = Some(new_id);
                        expect_fn_params = false;
                        continue;
                    }
                    if in_params {
                        param_depth += 1;
                    }
                    if in_decl {
                        decl_paren_depth += 1;
                    }
                }
                ")" => {
                    if in_params {
                        param_depth -= 1;
                        if param_depth == 0 {
                            in_params = false;
                            param_scope = None;
                        }
                    }
                    if in_decl && decl_paren_depth > 0 {
                        decl_paren_depth -= 1;
                    }
                }
                "=>" => {
                    if pending_fn.is_none() {
                        if i > 0 && tokens[i - 1].kind == TK::Ident {
                            let pname = txt(source, &tokens[i - 1]);
                            let new_id = scopes.len();
                            scopes.push(Scope {
                                parent: Some(cur),
                                is_function: true,
                                decls: HashSet::from([pname.to_string()]),
                            });
                            pending_fn = Some(new_id);
                            scopes[cur].decls.remove(pname);
                        }
                    }
                }
                "{" => {
                    if let Some(fn_id) = pending_fn.take() {
                        // Entering a function scope: save current decl state.
                        // Inner `var` declarations will reset in_decl, and we
                        // need to restore it when leaving this function scope.
                        decl_state_stack.push((
                            in_decl,
                            expect_decl_name,
                            decl_is_var,
                            decl_paren_depth,
                            decl_brace_depth,
                        ));
                        // Reset decl state for the new function scope
                        in_decl = false;
                        expect_decl_name = false;
                        decl_paren_depth = 0;
                        decl_brace_depth = 0;
                        stack.push(fn_id);
                        ts[i] = fn_id;
                    } else {
                        if in_decl {
                            decl_brace_depth += 1;
                        }
                        let new_id = scopes.len();
                        scopes.push(Scope {
                            parent: Some(cur),
                            is_function: false,
                            decls: HashSet::new(),
                        });
                        stack.push(new_id);
                        ts[i] = new_id;
                    }
                }
                "}" => {
                    if in_decl && decl_brace_depth > 0 {
                        decl_brace_depth -= 1;
                    }
                    if stack.len() > 1 {
                        let leaving_scope = stack.pop().unwrap();
                        // If leaving a function scope, restore saved decl state
                        if scopes[leaving_scope].is_function && !decl_state_stack.is_empty() {
                            let (
                                saved_in_decl,
                                saved_expect,
                                saved_is_var,
                                saved_paren,
                                saved_brace,
                            ) = decl_state_stack.pop().unwrap();
                            in_decl = saved_in_decl;
                            expect_decl_name = saved_expect;
                            decl_is_var = saved_is_var;
                            decl_paren_depth = saved_paren;
                            decl_brace_depth = saved_brace;
                        }
                    }
                    ts[i] = *stack.last().unwrap();
                }
                ";" => {
                    // Only end `in_decl` at the top level of the declaration.
                    // Semicolons inside object literals with function bodies
                    // (e.g., `var o = {f: function() { return 1; }}, x = 0;`)
                    // must NOT reset in_decl.
                    if decl_brace_depth == 0 && decl_paren_depth == 0 {
                        in_decl = false;
                        expect_decl_name = false;
                        decl_paren_depth = 0;
                    }
                }
                "," => {
                    // Re-enable decl name for: var a = 1, b = 2
                    // Only at the top level of the declaration (not inside
                    // parens, braces, or object literals).
                    if in_decl && decl_paren_depth == 0 && decl_brace_depth == 0 {
                        expect_decl_name = true;
                    }
                }
                _ => {}
            }
        }
    }

    ScopeInfo {
        scopes,
        token_scope: ts,
    }
}

// === Rename Computation ===

fn compute_renames(
    source: &str,
    tokens: &[Tok],
    si: &ScopeInfo,
    mangle_root: bool,
) -> Vec<HashMap<String, String>> {
    let mut renames: Vec<HashMap<String, String>> = vec![HashMap::new(); si.scopes.len()];

    // Pre-compute: for each scope, collect all ident names referenced in it and descendants
    let mut scope_refs: Vec<HashSet<String>> = vec![HashSet::new(); si.scopes.len()];
    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind == TK::Ident {
            scope_refs[si.token_scope[ti]].insert(txt(source, tok).to_string());
        }
    }
    // Propagate child refs up to parent scopes
    for sid in (0..si.scopes.len()).rev() {
        if let Some(pid) = si.scopes[sid].parent {
            let child_refs: Vec<String> = scope_refs[sid].iter().cloned().collect();
            for name in child_refs {
                scope_refs[pid].insert(name);
            }
        }
    }

    // Process scopes in order (parents before children)
    for (sid, scope) in si.scopes.iter().enumerate() {
        if !scope.is_function {
            continue;
        }
        // Skip root scope unless explicitly asked to mangle it
        // (e.g., when source is a scope-hoisted IIFE bundle).
        if sid == 0 && !mangle_root {
            continue;
        }
        let mut free_vars: HashSet<String> = HashSet::new();
        // Names referenced here but not declared → free vars (globals etc.)
        for name in &scope_refs[sid] {
            if !scope.decls.contains(name.as_str()) {
                free_vars.insert(name.clone());
            }
        }

        // Add ancestor rename values only for variables referenced in this scope tree
        let mut ancestor = scope.parent;
        while let Some(aid) = ancestor {
            for (orig_name, short_name) in &renames[aid] {
                if scope_refs[sid].contains(orig_name) {
                    free_vars.insert(short_name.clone());
                }
            }
            ancestor = si.scopes[aid].parent;
        }

        let mut skip_names = free_vars;
        skip_names.extend(scope.decls.iter().cloned());
        let mut gen = NameGen::new(&skip_names);
        let mut decls: Vec<&String> = scope.decls.iter().collect();
        decls.sort();
        for name in decls {
            if is_reserved(name) {
                continue;
            }
            let short = gen.next_name();
            if short.len() < name.len() {
                renames[sid].insert(name.clone(), short);
            }
        }
    }

    renames
}

fn apply_renames(
    source: &str,
    tokens: &[Tok],
    token_scope: &[usize],
    scopes: &[Scope],
    renames: &[HashMap<String, String>],
) -> String {
    let mut repls: Vec<(usize, usize, String)> = Vec::new();

    for (ti, tok) in tokens.iter().enumerate() {
        if tok.kind != TK::Ident {
            continue;
        }
        let name = txt(source, tok);
        // Skip property access (after .)
        if ti > 0 && tokens[ti - 1].kind == TK::Punct && txt(source, &tokens[ti - 1]) == "." {
            continue;
        }
        // Skip object literal keys: { key: value }
        if ti + 1 < tokens.len()
            && tokens[ti + 1].kind == TK::Punct
            && txt(source, &tokens[ti + 1]) == ":"
            && ti > 0
            && tokens[ti - 1].kind == TK::Punct
        {
            let prev = txt(source, &tokens[ti - 1]);
            if prev == "{" || prev == "," {
                continue;
            }
        }
        // Resolve through scope chain
        let mut sid = token_scope[ti];
        loop {
            if let Some(new_name) = renames[sid].get(name) {
                repls.push((tok.start, tok.end, new_name.clone()));
                break;
            }
            if scopes[sid].decls.contains(name) {
                break; // Declared here but not renamed
            }
            match scopes[sid].parent {
                Some(p) => sid = p,
                None => break, // Global
            }
        }
    }

    // Apply in reverse order
    repls.sort_by(|a, b| b.0.cmp(&a.0));
    let mut result = source.as_bytes().to_vec();
    for (start, end, new_name) in &repls {
        let nb = new_name.as_bytes();
        result.splice(start..end, nb.iter().cloned());
    }
    String::from_utf8(result).unwrap_or_else(|_| source.to_string())
}

// === Name Generator ===

struct NameGen {
    counter: usize,
    skip: HashSet<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
impl NameGen {
    fn new(skip: &HashSet<String>) -> Self {
        let mut all_skip = skip.clone();
        for kw in RESERVED {
            all_skip.insert(kw.to_string());
        }
        Self {
            counter: 0,
            skip: all_skip,
        }
    }

    fn next_name(&mut self) -> String {
        loop {
            let name = gen_name(self.counter);
            self.counter += 1;
            if !self.skip.contains(&name) {
                return name;
            }
        }
    }
}

fn gen_name(mut n: usize) -> String {
    const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";
    const REST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$";
    let mut name = vec![FIRST[n % FIRST.len()]];
    n /= FIRST.len();
    while n > 0 {
        n -= 1;
        name.push(REST[n % REST.len()]);
        n /= REST.len();
    }
    String::from_utf8(name).unwrap()
}

fn is_reserved(name: &str) -> bool {
    RESERVED.contains(&name)
}

const RESERVED: &[&str] = &[
    // Keywords
    "break",
    "case",
    "catch",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "finally",
    "for",
    "function",
    "if",
    "in",
    "instanceof",
    "new",
    "return",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "class",
    "const",
    "enum",
    "export",
    "extends",
    "import",
    "super",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "async",
    "await",
    "of",
    // Literals
    "null",
    "undefined",
    "true",
    "false",
    "NaN",
    "Infinity",
    // Module system — `require` kept reserved as it appears as a global in
    // many environments; `module` and `exports` are intentionally NOT
    // reserved so the mangler can rename them when they appear as function
    // parameters inside per-module IIFE wrappers (e.g., the scope-hoisted
    // `!function(module,exports,require){...}()` format), turning the
    // 7-byte `exports` parameter into a 1-byte short name.
    "require",
    "arguments",
    // Common globals (safety net)
    "window",
    "document",
    "console",
    "navigator",
    "location",
    "history",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "requestAnimationFrame",
    "cancelAnimationFrame",
    "Promise",
    "Symbol",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "Proxy",
    "Reflect",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Function",
    "RegExp",
    "Date",
    "Error",
    "TypeError",
    "RangeError",
    "SyntaxError",
    "ReferenceError",
    "Math",
    "JSON",
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
    "encodeURIComponent",
    "decodeURIComponent",
    "encodeURI",
    "decodeURI",
    "eval",
    "globalThis",
    "self",
    "queueMicrotask",
    "structuredClone",
    "fetch",
    "AbortController",
    "URL",
    "URLSearchParams",
    "TextEncoder",
    "TextDecoder",
    "Blob",
    "File",
    "FormData",
    "Headers",
    "Request",
    "Response",
    "ReadableStream",
    "WritableStream",
    "TransformStream",
    "Event",
    "EventTarget",
    "CustomEvent",
    "MutationObserver",
    "IntersectionObserver",
    "ResizeObserver",
    "performance",
    "crypto",
    "atob",
    "btoa",
    "Node",
    "Element",
    "HTMLElement",
    "SVGElement",
    "DocumentFragment",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_var_mangling() {
        let src = "function f() { var longName = 1; return longName + 2; }";
        let out = mangle_variables(src);
        assert!(
            !out.contains("longName"),
            "should mangle longName, got: {}",
            out
        );
        assert!(
            out.contains("return a + 2") || out.contains("return a+2"),
            "should use short name, got: {}",
            out
        );
    }

    #[test]
    fn test_param_mangling() {
        let src = "function add(first, second) { return first + second; }";
        let out = mangle_variables(src);
        assert!(!out.contains("first"), "should mangle first, got: {}", out);
        assert!(
            !out.contains("second"),
            "should mangle second, got: {}",
            out
        );
    }

    #[test]
    fn test_mangling_does_not_collide_with_existing_short_param() {
        let src = "\"use strict\";function batchedUpdates(callback,a){return callback(a);}";
        let out = mangle_variables(src);
        assert!(
            !out.contains("function batchedUpdates(a,a)"),
            "mangler must not create duplicate parameter names, got: {}",
            out
        );
    }

    #[test]
    fn test_globals_preserved() {
        let src = "function f() { console.log(document.title); }";
        let out = mangle_variables(src);
        assert!(
            out.contains("console"),
            "console should be preserved, got: {}",
            out
        );
        assert!(
            out.contains("document"),
            "document should be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_property_access_preserved() {
        let src = "function f() { var obj = {}; obj.longProp = 1; return obj.longProp; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("longProp"),
            "property should be preserved, got: {}",
            out
        );
        assert!(
            !out.contains("var obj"),
            "local var should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_module_level_not_mangled() {
        let src = "var topLevel = 1; function f() { var local = topLevel; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("topLevel"),
            "module-level should not be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_iife_vars_mangled_with_root() {
        // Scope-hoisted bundle: outer IIFE wraps everything.
        // !function(module,exports,require){var workInProgress = null;...}(...)
        // workInProgress is declared inside a function scope and should be mangled.
        let src = "!function(module,exports,require){var workInProgress=null;var executionContext=0;workInProgress=executionContext}(_m0,_m0.exports,_r)";
        let out = mangle_variables(src);
        // These should already be mangled since !function creates a function scope
        assert!(
            !out.contains("workInProgress"),
            "workInProgress should be mangled in IIFE, got: {}",
            out
        );
    }

    #[test]
    fn test_outer_iife_vars_mangled_with_root() {
        // The outer IIFE of a scope-hoisted bundle
        let src = "(function(){var _m0={exports:{}};var longVarName=1;longVarName=2})()";
        let out = mangle_variables_with_root(src);
        assert!(
            !out.contains("longVarName"),
            "IIFE var should be mangled with root, got: {}",
            out
        );
    }

    #[test]
    fn test_iife_closure_vars_mangled() {
        // Real pattern: var declared in !function wrapper, referenced in nested functions.
        // The mangler should rename these since they're function-local (closure vars are ok to mangle
        // as long as ALL references are renamed consistently).
        let src = r#"!function(module,exports,require){var workInProgress=null;var executionContext=0;function performWork(){if(workInProgress!==null){executionContext=1}}function commitRoot(){workInProgress=null;executionContext=0}exports.performWork=performWork;exports.commitRoot=commitRoot}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("workInProgress"),
            "workInProgress should be mangled in !function IIFE, got: {}",
            out
        );
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_with_object_literal() {
        // Bug: comma inside object literal was incorrectly re-triggering
        // expect_decl_name, causing subsequent var names to be missed.
        let src =
            r#"!function(){var config={x:1,y:2},longVarName=3;function f(){return longVarName}}()"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("longVarName"),
            "longVarName after object literal should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_with_object_containing_functions() {
        let src = r#"!function(){var dispatcher={getCacheForType:function(a){return a},cacheSignal:function(){return 1}},executionContext=0,workInProgressRoot=null;function f(){workInProgressRoot=1;executionContext=2}}()"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("executionContext"),
            "executionContext after object-with-functions should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_after_minify() {
        // Simulate the real pipeline: minify first, then mangle.
        // The minifier may alter token boundaries that confuse scope analysis.
        let raw = r#"!function(){
  var dispatcher = {
    getCacheForType: function (a) { return a; },
    cacheSignal: function () { return 1; }
  },
  executionContext = 0,
  workInProgressRoot = null;
  function f() { workInProgressRoot = 1; executionContext = 2; }
}()"#;
        let minified = crate::bundler::minify::minify_js(raw, &[]);
        let out = mangle_variables(&minified);
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled after minify+mangle, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled after minify+mangle, got: {}",
            out
        );
    }

    #[test]
    fn test_multi_var_with_ternary() {
        // React pattern: var PossiblyWeakMap = "function" === typeof WeakMap ? WeakMap : Map,
        //                    executionContext = 0, workInProgressRoot = null;
        let src = r#"!function(){var PossiblyWeakMap="function"===typeof WeakMap?WeakMap:Map,executionContext=0,workInProgressRoot=null;function f(){workInProgressRoot=1;executionContext=2}}()"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_many_module_scope_vars_mangled() {
        // React DOM pattern: many long var names at !function top level,
        // all referenced in nested functions. All should be mangled.
        let src = r#"!function(a,b,c){var workInProgressRoot=null;var workInProgressRootRenderLanes=0;var executionContext=0;var workInProgressSuspendedReason=0;function performWork(){workInProgressRoot=1;workInProgressRootRenderLanes=2;executionContext=3}function check(){return workInProgressSuspendedReason}performWork();check()}(_m0,_m0.exports,_r)"#;
        let out = mangle_variables(src);
        assert!(
            !out.contains("workInProgressRoot"),
            "workInProgressRoot should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressRootRenderLanes"),
            "workInProgressRootRenderLanes should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("executionContext"),
            "executionContext should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("workInProgressSuspendedReason"),
            "workInProgressSuspendedReason should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_string_content_preserved() {
        let src = r#"function f() { var x = "longVariableName"; return x; }"#;
        let out = mangle_variables(src);
        assert!(
            out.contains("\"longVariableName\""),
            "string should be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_name_generator() {
        let skip = HashSet::new();
        let mut gen = NameGen::new(&skip);
        assert_eq!(gen.next_name(), "a");
        assert_eq!(gen.next_name(), "b");
    }

    #[test]
    fn test_gen_name_sequence() {
        assert_eq!(gen_name(0), "a");
        assert_eq!(gen_name(25), "z");
        assert_eq!(gen_name(26), "A");
        assert_eq!(gen_name(51), "Z");
        assert_eq!(gen_name(52), "_");
        assert_eq!(gen_name(53), "$");
    }

    #[test]
    fn test_reserved_skipped() {
        let mut skip = HashSet::new();
        skip.insert("a".to_string());
        let mut gen = NameGen::new(&skip);
        let first = gen.next_name();
        assert_ne!(first, "a", "should skip 'a'");
        assert_eq!(first, "b");
    }

    #[test]
    fn test_wrapper_function_mangling() {
        let src = "__jet__.define(2,function(require,module,exports){var longVarName=0;function foo(bar){var innerLong=bar;return innerLong+longVarName;}})";
        let out = mangle_variables(src);
        eprintln!("wrapper output: {}", out);
        assert!(
            !out.contains("longVarName"),
            "wrapper var should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("innerLong"),
            "inner var should be mangled, got: {}",
            out
        );
        assert!(out.contains("require"), "require preserved, got: {}", out);
        // `module` and `exports` are no longer reserved — they should be mangled
        // to short names when used as function parameters.
        // The function signature must not contain the original long names as params.
        assert!(
            !out.contains(",module,"),
            "module param should be mangled, got: {}",
            out
        );
        // `exports` as a param appears as `,exports)` in the original; it should be renamed.
        assert!(
            !out.contains(",exports)"),
            "exports param should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_scope_hoisted_module_mangling() {
        // Simulate a scope-hoisted Phase-1 module IIFE: exports/module are params
        // and should be mangled to short names.
        let src = "!function(module,exports,require){var workInProgress=null;exports.render=workInProgress;}(_m0,_m0.exports,_r)";
        let out = mangle_variables(src);
        eprintln!("scope-hoisted output: {}", out);
        // workInProgress is a local var — must be mangled
        assert!(
            !out.contains("workInProgress"),
            "workInProgress should be mangled, got: {}",
            out
        );
        // `exports` as a function parameter must be renamed — the signature
        // `(module,exports,require)` must be gone. Note that `_m0.exports`
        // (property access) is NOT renamed, so we check the param list, not
        // the whole output.
        assert!(
            !out.contains(",exports,") && !out.contains("(exports,") && !out.contains(",exports)"),
            "exports param should be mangled (only .exports property access remains), got: {}",
            out
        );
        // module is a parameter — must be mangled (no longer appears as standalone `module,`)
        assert!(
            !out.contains("(module,") && !out.contains(",module,") && !out.contains(",module)"),
            "module param should be mangled, got: {}",
            out
        );
        // require is still reserved — must NOT be mangled
        assert!(
            out.contains("require"),
            "require should be preserved, got: {}",
            out
        );
        // The property access `.exports` (in the argument `_m0.exports`) must be preserved
        assert!(
            out.contains(".exports"),
            ".exports property access preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_root_scope_mangle_order_is_deterministic() {
        let src = r#"(function(){var zebra={exports:{}},alpha={exports:{}},middle={exports:{}};function read(which){switch(which){case 0:return zebra.exports;case 1:return alpha.exports;default:return middle.exports}}return read(0)})()"#;
        let first = mangle_variables_with_root(src);
        for _ in 0..32 {
            assert_eq!(mangle_variables_with_root(src), first);
        }
    }

    #[test]
    fn test_nested_function_mangling() {
        let src = "function outer(){var longOuter=1;function inner(){var longInner=2;return longInner;}return longOuter;}";
        let out = mangle_variables(src);
        eprintln!("nested output: {}", out);
        assert!(
            !out.contains("longOuter"),
            "should mangle longOuter, got: {}",
            out
        );
        assert!(
            !out.contains("longInner"),
            "should mangle longInner, got: {}",
            out
        );
    }

    #[test]
    fn test_keywords_not_generated() {
        // "in" is a keyword — name generator should skip it
        let skip = HashSet::new();
        let mut gen = NameGen::new(&skip);
        let mut names = Vec::new();
        for _ in 0..100 {
            names.push(gen.next_name());
        }
        assert!(
            !names.contains(&"in".to_string()),
            "should not generate keyword 'in'"
        );
        assert!(
            !names.contains(&"do".to_string()),
            "should not generate keyword 'do'"
        );
        assert!(
            !names.contains(&"if".to_string()),
            "should not generate keyword 'if'"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // UTF-8 multi-byte safety tests (issue #904)
    //
    // The tokenizer operates on raw bytes (`source.as_bytes()`) and only
    // matches ASCII identifier characters, so multi-byte UTF-8 sequences
    // pass through as opaque Punct tokens.  `apply_renames` reconstructs
    // the output via `result.splice(byte_start..byte_end, ...)` where
    // offsets come from the byte-level scan — no char-index-as-byte-offset
    // bug is possible.  These tests verify that.
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_utf8_string_content_unchanged() {
        // ✓ is 3-byte UTF-8; the string literal must survive intact.
        // Use a multi-char name so the mangler actually renames it.
        let src = r#"function f() { var checkResult = "✓ ok"; return checkResult; }"#;
        let out = mangle_variables(src);
        assert!(
            out.contains("\"✓ ok\""),
            "UTF-8 string must be unchanged, got: {}",
            out
        );
        // Local `checkResult` should be mangled to a shorter name
        assert!(
            !out.contains("checkResult"),
            "local checkResult should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_utf8_emoji_in_string_preserved() {
        // 🎉 is 4-byte UTF-8
        let src = r#"function f() { var msg = "Hello 🎉"; return msg; }"#;
        let out = mangle_variables(src);
        assert!(
            out.contains("\"Hello 🎉\""),
            "emoji string must be preserved, got: {}",
            out
        );
    }

    #[test]
    fn test_utf8_cjk_in_string_preserved() {
        // Use a multi-char name so the mangler actually renames it.
        let src = "function f() { var strVal = '日本語テスト'; return strVal; }";
        let out = mangle_variables(src);
        assert!(
            out.contains("'日本語テスト'"),
            "CJK string must be preserved, got: {}",
            out
        );
        // `strVal` should be mangled to a shorter name
        assert!(
            !out.contains("strVal"),
            "local strVal should be mangled, got: {}",
            out
        );
    }

    #[test]
    fn test_utf8_mixed_code_and_strings() {
        // Multi-byte chars before and after identifiers that get renamed
        let src = "function f() { var longName = '✓'; var other = longName + '日'; return other; }";
        let out = mangle_variables(src);
        // Strings preserved
        assert!(out.contains("'✓'"), "✓ string preserved, got: {}", out);
        assert!(out.contains("'日'"), "日 string preserved, got: {}", out);
        // Identifiers mangled
        assert!(
            !out.contains("longName"),
            "longName should be mangled, got: {}",
            out
        );
        assert!(
            !out.contains("other"),
            "other should be mangled, got: {}",
            out
        );
    }
}
// CODEGEN-END
