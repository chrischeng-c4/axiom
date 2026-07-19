// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
/// Import/Export detection using Tree-sitter.
///
/// Also provides `apply_alias` — a lightweight pre-processor that substitutes
/// module path aliases (e.g. `@/components/Foo` → `./src/components/Foo`)
/// before the specifier is handed to the Node.js resolver.  This mirrors
/// what Vite does: alias resolution happens in the module graph construction
/// step, before any `node_modules` lookup.
use anyhow::Result;
use tree_sitter::{Node, Parser};

/// Import/export information extracted from a module
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleImports {
    pub static_imports: Vec<ImportDeclaration>,
    pub dynamic_imports: Vec<String>,
    pub exports: Vec<ExportDeclaration>,
}

/// Static import declaration
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDeclaration {
    pub source: String,
    pub kind: ImportKind,
}

/// Kind of import
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Default,
    Named,
    Namespace,
    SideEffect,
}

/// Export declaration
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ExportDeclaration {
    pub kind: ExportKind,
    pub source: Option<String>,
}

/// Kind of export
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportKind {
    Named,
    Default,
    All,
}

/// Extract imports from JavaScript/TypeScript source code
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn extract_imports(source: &str, is_typescript: bool) -> Result<ModuleImports> {
    extract_imports_with_tree(source, is_typescript).map(|(imports, _tree)| imports)
}

/// Like [`extract_imports`] but also hands back the parsed tree-sitter tree.
///
/// jet otherwise tree-sitter-parses every module twice — once here during
/// graph construction to discover imports, and again in the module transform
/// for codegen. For a plain-JS module (`.js`/`.cjs`/`.mjs`) whose source the
/// transform does not rewrite first, this tree (parsed with the JS grammar) is
/// byte-for-byte the same parse the transform would redo, so the caller can
/// stash it and skip the second parse. TS/TSX/JSX trees are NOT reusable: their
/// source is rewritten before the module transform parses it.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn extract_imports_with_tree(
    source: &str,
    is_typescript: bool,
) -> Result<(ModuleImports, tree_sitter::Tree)> {
    let mut parser = Parser::new();

    let language = if is_typescript {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    } else {
        tree_sitter_javascript::LANGUAGE.into()
    };

    parser.set_language(&language)?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse source"))?;

    let root = tree.root_node();

    let mut imports = ModuleImports {
        static_imports: Vec::new(),
        dynamic_imports: Vec::new(),
        exports: Vec::new(),
    };

    extract_from_node(source, &root, &mut imports);

    Ok((imports, tree))
}

/// Recursively extract imports/exports from AST node
fn extract_from_node(source: &str, node: &Node, imports: &mut ModuleImports) {
    match node.kind() {
        "import_statement" => {
            if let Some(import_decl) = parse_import_statement(source, node) {
                imports.static_imports.push(import_decl);
            }
        }

        "call_expression" => {
            if is_dynamic_import(node) {
                if let Some(specifier) = extract_dynamic_import(source, node) {
                    imports.dynamic_imports.push(specifier);
                }
            } else if is_require_call(source, node) {
                if let Some(specifier) = extract_require_specifier(source, node) {
                    imports.static_imports.push(ImportDeclaration {
                        source: specifier,
                        kind: ImportKind::Default,
                    });
                }
            }
        }

        "export_statement" => {
            if let Some(export_decl) = parse_export_statement(source, node) {
                // Re-exports with a source need to be tracked as static imports too
                if let Some(ref src) = export_decl.source {
                    imports.static_imports.push(ImportDeclaration {
                        source: src.clone(),
                        kind: ImportKind::Named,
                    });
                }
                imports.exports.push(export_decl);
            }
        }

        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_from_node(source, &child, imports);
    }
}

fn parse_import_statement(source: &str, node: &Node) -> Option<ImportDeclaration> {
    let source_node = find_child_by_kind(node, "string")?;
    let source_text = node_text(source, &source_node);
    let import_source = strip_quotes(&source_text);

    let kind = determine_import_kind(node);

    Some(ImportDeclaration {
        source: import_source,
        kind,
    })
}

fn determine_import_kind(node: &Node) -> ImportKind {
    if let Some(import_clause) = find_child_by_kind(node, "import_clause") {
        if find_child_by_kind(&import_clause, "identifier").is_some() {
            return ImportKind::Default;
        }
        if find_child_by_kind(&import_clause, "namespace_import").is_some() {
            return ImportKind::Namespace;
        }
        return ImportKind::Named;
    }
    ImportKind::SideEffect
}

fn is_dynamic_import(node: &Node) -> bool {
    if let Some(function) = find_child_by_kind(node, "import") {
        return function.kind() == "import";
    }
    false
}

fn extract_dynamic_import(source: &str, node: &Node) -> Option<String> {
    let args = find_child_by_kind(node, "arguments")?;
    let string_node = find_child_by_kind(&args, "string")?;
    let source_text = node_text(source, &string_node);
    Some(strip_quotes(&source_text))
}

fn is_require_call(source: &str, node: &Node) -> bool {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if let Some(function) = children.first() {
        if function.kind() == "identifier" {
            let function_name = node_text(source, function);
            return function_name == "require";
        }
    }
    false
}

fn extract_require_specifier(source: &str, node: &Node) -> Option<String> {
    let args = find_child_by_kind(node, "arguments")?;

    let mut cursor = args.walk();
    let children: Vec<_> = args.children(&mut cursor).collect();

    for child in children {
        if child.kind() == "string" {
            let source_text = node_text(source, &child);
            let specifier = strip_quotes(&source_text);

            if specifier.contains(".development.") || specifier.contains("-development.") {
                tracing::debug!("Skipping development build: {}", specifier);
                return None;
            }

            return Some(specifier);
        }
    }

    None
}

fn parse_export_statement(source: &str, node: &Node) -> Option<ExportDeclaration> {
    // Check for star re-export: export * from "./foo"
    if find_child_by_kind(node, "*").is_some() {
        let source_node = find_child_by_kind(node, "string");
        let source_val = source_node.map(|n| strip_quotes(&node_text(source, &n)));
        return Some(ExportDeclaration {
            kind: ExportKind::All,
            source: source_val,
        });
    }

    // Named re-export: export { X } from "./X" or local export { X }
    if find_child_by_kind(node, "export_clause").is_some() {
        let source_node = find_child_by_kind(node, "string");
        let source_val = source_node.map(|n| strip_quotes(&node_text(source, &n)));
        return Some(ExportDeclaration {
            kind: ExportKind::Named,
            source: source_val,
        });
    }

    if node_text(source, node).contains("export default") {
        return Some(ExportDeclaration {
            kind: ExportKind::Default,
            source: None,
        });
    }

    Some(ExportDeclaration {
        kind: ExportKind::Named,
        source: None,
    })
}

fn find_child_by_kind<'a>(node: &'a Node, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    children.into_iter().find(|child| child.kind() == kind)
}

fn node_text(source: &str, node: &Node) -> String {
    source[node.byte_range()].to_string()
}

fn strip_quotes(s: &str) -> String {
    s.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

// ─── String-scan fast path for crawl-time extraction (#1997) ──────────────

/// Cheap byte-level alternative to [`extract_imports_with_tree`] for the
/// module-graph crawl (`Bundler::prefetch_one_module`,
/// `Bundler::runtime_static_imports`): produces the identical
/// `ModuleImports` shape without a tree-sitter parse, for the large
/// majority of real-world modules whose import/export/require/dynamic-import
/// statements are plain, unambiguous text. Returns `None` — the caller must
/// fall back to [`extract_imports_with_tree`] — for any source this scan
/// cannot classify soundly; a missed edge would be a missing module at
/// runtime, so every bail below is deliberately conservative.
///
/// Eligibility / fallback rules (anything not listed here also bails):
/// - An unterminated line/block comment, plain string, or template
///   literal falls back (the file is presumably truncated/malformed, or
///   this scan mis-detected a construct boundary).
/// - A template literal nested inside another template's `${}`
///   substitution falls back (accepted complexity cutoff — see WI #1997).
/// - A plain `'...'`/`"..."` string whose content would need to span a
///   raw newline falls back (not a legal single-line string; also covers
///   the rare backslash-newline continuation form).
/// - Any single physical line longer than 4000 bytes falls back (a
///   minified/compacted `node_modules` ESM signal — a real single
///   giant line can cram many statements together, invisible to a
///   line-oriented scan).
/// - `import.meta.*` needs no special handling: neither scanner below
///   matches `import` followed by `.` (the statement scanner requires a
///   space/quote/brace/star immediately after `import`; the call scanner
///   requires an immediate, optionally-whitespace-separated `(`), exactly
///   mirroring `extract_from_node`, which also never matches it (it is
///   neither an `import_statement`, a `call_expression` recognized by
///   `is_dynamic_import`/`is_require_call`, nor an `export_statement`).
/// - A non-literal dynamic-import/require argument (template literal,
///   identifier, expression) is silently skipped, not a bail — matching
///   `is_dynamic_import`/`extract_dynamic_import`'s and
///   `is_require_call`/`extract_require_specifier`'s existing behavior of
///   only recognizing a literal string argument.
/// - `require.resolve(...)`, `(0, require)('x')`, and other non-bare-call
///   forms are not recognized — matching `is_require_call`'s own
///   first-child-must-be-a-bare-`identifier` restriction.
/// - Regex-literal-aware lexing is deliberately NOT implemented: a
///   character class containing `/*`/`//` (e.g. `/[/*]/`) could in theory
///   fool the comment detector into swallowing subsequent real code. This
///   is an accepted, documented residual gap (WI #1997) rather than added
///   complexity; the differential harness is the empirical backstop.
/// - `declare`/ambient TypeScript constructs are not specially modeled,
///   but need no special handling either: tree-sitter's own walk has no
///   special-casing for ambient context, so ordinary line-based scanning
///   already agrees with it.
/// - An export's re-export `source` (and the extra `static_imports` edge
///   that comes with it) is recognized only via a genuine whole-word
///   `from` keyword immediately before the specifier's opening quote
///   ([`from_clause_specifier`]), never a blind trailing-quote scan —
///   `export const UNMOUNTED = 'unmounted';` (a real line from
///   `react-transition-group`'s `Transition.js`) has no `from` clause at
///   all and must not fabricate a `source: Some("unmounted")` re-export
///   edge the crawl would then fail to resolve as a package.
/// - An import-attributes clause (`from './data.json' with { type:
///   'json' }`, or the older `assert { ... }` form) never contaminates
///   the specifier: [`from_clause_specifier`] takes the FIRST quoted
///   string after `from`, not the last quoted string on the line.
/// - `export * as <ident> from '<spec>'` (aliased namespace re-export)
///   falls back: this is not a soundness gap in the scan itself (a
///   structural check would classify it correctly), but tree-sitter's own
///   `parse_export_statement` cannot see this shape's `*` as an
///   `export_statement` direct child (wrapped in its own `namespace_export`
///   node instead) and has therefore always silently dropped its crawl
///   edge — see [`is_aliased_namespace_reexport`]'s doc comment. Bailing
///   here keeps this WI perf-only rather than also being a de facto fix to
///   that pre-existing, out-of-scope tree-sitter gap.
///
/// `JET_NO_FAST_IMPORT_SCAN=1` forces every module through the
/// tree-sitter path unconditionally (this function always returns `None`).
pub fn extract_imports_fast(source: &str) -> Option<ModuleImports> {
    if std::env::var_os("JET_NO_FAST_IMPORT_SCAN").is_some() {
        return None;
    }

    let line_view = lex_scan(source)?;

    // Statement-level entries (import/export, byte-offset tagged) and
    // call-level entries (require/dynamic-import, byte-offset tagged) are
    // scanned independently, then merged by offset below so
    // `static_imports`' relative order matches the single interleaved
    // depth-first order `extract_from_node`'s walk would produce --
    // `build_graph`'s serial replay pushes crawl-queue entries in
    // `static_imports` vector order, which can affect module-id
    // assignment and output ordering.
    let mut static_imports: Vec<(usize, ImportDeclaration)> = Vec::new();
    let mut exports: Vec<ExportDeclaration> = Vec::new();

    for (offset, line) in logical_top_level_lines(&line_view) {
        let trimmed = line.trim();
        if trimmed.starts_with("import") {
            if let Some(decl) = classify_import_line(trimmed) {
                static_imports.push((offset, decl));
            }
        } else if trimmed.starts_with("export") {
            // `export * as ns from '...'` forces a whole-module bail --
            // see `is_aliased_namespace_reexport`'s doc comment.
            if is_aliased_namespace_reexport(trimmed) {
                return None;
            }
            if let Some((export_decl, extra_import)) = classify_export_line(trimmed) {
                if let Some(extra) = extra_import {
                    static_imports.push((offset, extra));
                }
                exports.push(export_decl);
            }
        }
    }

    let (call_requires, mut call_dynamics) = scan_calls(&line_view);
    static_imports.extend(call_requires);
    static_imports.sort_by_key(|(offset, _)| *offset);

    call_dynamics.sort_by_key(|(offset, _)| *offset);

    Some(ModuleImports {
        static_imports: static_imports.into_iter().map(|(_, decl)| decl).collect(),
        dynamic_imports: call_dynamics.into_iter().map(|(_, spec)| spec).collect(),
        exports,
    })
}

/// Length-preserving lexical pre-pass: replaces every line comment, block
/// comment, and template-literal span with ASCII spaces (real `\n` bytes
/// always preserved), leaving plain `'...'`/`"..."` string content
/// verbatim so the statement/call scanners below can read real specifier
/// text. Byte offsets into the returned string always match offsets into
/// `source` (every branch consumes and emits the same number of bytes),
/// which is what lets the caller merge-sort statement- and call-level
/// entries by position. Returns `None` (bail) on anything this pass
/// cannot classify soundly — see [`extract_imports_fast`]'s doc comment
/// for the exact list.
fn lex_scan(source: &str) -> Option<String> {
    // Cheap minified/compacted-source signal, checked up front so an
    // obviously-ineligible file never pays for the byte scan below.
    const MAX_LINE_LEN: usize = 4000;
    if source.lines().any(|line| line.len() > MAX_LINE_LEN) {
        return None;
    }

    let bytes = source.as_bytes();
    let n = bytes.len();
    let mut out: Vec<u8> = Vec::with_capacity(n);
    let mut i = 0usize;
    while i < n {
        match bytes[i] {
            b'/' if i + 1 < n && bytes[i + 1] == b'/' => {
                let start = i;
                i += 2;
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
                out.resize(out.len() + (i - start), b' ');
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                let mut j = i + 2;
                let mut closed = false;
                while j + 1 < n {
                    if bytes[j] == b'*' && bytes[j + 1] == b'/' {
                        j += 2;
                        closed = true;
                        break;
                    }
                    j += 1;
                }
                if !closed {
                    return None;
                }
                for k in i..j {
                    out.push(if bytes[k] == b'\n' { b'\n' } else { b' ' });
                }
                i = j;
            }
            b'\'' | b'"' => {
                let quote = bytes[i];
                let mut j = i + 1;
                let mut closed = false;
                while j < n {
                    match bytes[j] {
                        b'\\' if j + 1 < n => {
                            if bytes[j + 1] == b'\n' {
                                return None; // backslash-newline continuation -- bail
                            }
                            j += 2;
                        }
                        b'\n' => break, // unterminated on this line -- bail below
                        c if c == quote => {
                            j += 1;
                            closed = true;
                            break;
                        }
                        _ => j += 1,
                    }
                }
                if !closed {
                    return None;
                }
                out.extend_from_slice(&bytes[i..j]);
                i = j;
            }
            b'`' => match scan_template_span(bytes, i) {
                Some(end) => {
                    for k in i..end {
                        out.push(if bytes[k] == b'\n' { b'\n' } else { b' ' });
                    }
                    i = end;
                }
                None => return None,
            },
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// Finds the end (one past the closing backtick) of a template literal
/// starting at `bytes[start] == b'`'`, stepping over `${...}`
/// substitutions (which may contain their own plain strings) via
/// brace-depth tracking. Returns `None` — bail, the caller routes the
/// whole module to the tree-sitter fallback — the moment it hits a
/// template literal nested inside a substitution, an unterminated
/// string/substitution, or EOF before the closing backtick.
fn scan_template_span(bytes: &[u8], start: usize) -> Option<usize> {
    let n = bytes.len();
    let mut i = start + 1;
    loop {
        if i >= n {
            return None;
        }
        match bytes[i] {
            b'\\' if i + 1 < n => i += 2,
            b'`' => return Some(i + 1),
            b'$' if i + 1 < n && bytes[i + 1] == b'{' => {
                i += 2;
                let mut depth = 1usize;
                while depth > 0 {
                    if i >= n {
                        return None;
                    }
                    match bytes[i] {
                        b'{' => {
                            depth += 1;
                            i += 1;
                        }
                        b'}' => {
                            depth -= 1;
                            i += 1;
                        }
                        b'\'' | b'"' => {
                            let quote = bytes[i];
                            let mut j = i + 1;
                            let mut closed = false;
                            while j < n {
                                match bytes[j] {
                                    b'\\' if j + 1 < n => j += 2,
                                    b'\n' => break,
                                    c if c == quote => {
                                        j += 1;
                                        closed = true;
                                        break;
                                    }
                                    _ => j += 1,
                                }
                            }
                            if !closed {
                                return None;
                            }
                            i = j;
                        }
                        b'`' => return None, // nested template literal -- bail
                        _ => i += 1,
                    }
                }
            }
            _ => i += 1,
        }
    }
}

/// Splits `s` into `(byte_offset, line)` pairs (offsets index into `s`
/// itself), the offset-aware counterpart of `str::lines` used so
/// statement-level entries can be merge-sorted against call-level entries
/// by source position.
fn lines_with_offsets(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in s.split('\n') {
        out.push((offset, line));
        offset += line.len() + 1;
    }
    out
}

fn brace_depth_delta(s: &str) -> i32 {
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
    }
    depth
}

/// Groups `line_view` into `(byte_offset, logical_line)` pairs for every
/// `import`/`export`-statement-initial physical line, joining a wrapped
/// named-binding clause (prettier's default formatting for 3+ named
/// bindings — `import {\n  A,\n  B,\n} from '...';`) into one logical
/// line first, generalizing `tree_shake::logical_import_lines`'s
/// technique to also cover `export` (which it does not handle) and to
/// use brace-depth-closure rather than "a specifier eventually appears"
/// as the termination signal (needed because a bare, no-`from` clause
/// never has a specifier to terminate on, and unbounded accumulation
/// while chasing one risks swallowing subsequent real statements — see
/// [`needs_clause_join`]). Non-statement lines are dropped; the caller
/// only wants statement-initial lines.
fn logical_top_level_lines(line_view: &str) -> Vec<(usize, String)> {
    let raw = lines_with_offsets(line_view);
    let mut out: Vec<(usize, String)> = Vec::new();
    let mut idx = 0usize;
    while idx < raw.len() {
        let (line_offset, line) = raw[idx];
        let trimmed = line.trim_start();
        let is_import = trimmed.starts_with("import ")
            || trimmed.starts_with("import{")
            || trimmed.starts_with("import'")
            || trimmed.starts_with("import\"")
            || trimmed.starts_with("import*");
        let is_export = trimmed.starts_with("export ") || trimmed.starts_with("export{");
        if !is_import && !is_export {
            idx += 1;
            continue;
        }

        let mut joined = trimmed.trim_end().to_string();
        let mut consumed = 1usize;
        if needs_clause_join(&joined, is_import) {
            const MAX_JOIN_LINES: usize = 4096;
            let mut depth = brace_depth_delta(&joined);
            while depth > 0 && consumed < MAX_JOIN_LINES && idx + consumed < raw.len() {
                let next = raw[idx + consumed].1.trim();
                if !next.is_empty() {
                    joined.push(' ');
                    joined.push_str(next);
                    depth += brace_depth_delta(next);
                }
                consumed += 1;
            }
        }
        out.push((line_offset, joined));
        idx += consumed;
    }
    out
}

/// Whether a statement-initial line might still need subsequent physical
/// lines joined onto it before it is classifiable. Import clauses are
/// always safe to brace-depth-track unconditionally — nothing but `from
/// '<specifier>'` and a semicolon can ever follow an import clause, so an
/// open `{` (including the `import Default, { Named } from '...'` combo
/// shape, whose clause text does not itself start with `{`) can never be
/// confused with an unrelated function/class body. Export has no
/// default+named combo form (`export default` is always alone), so a
/// narrower "clause literally starts with `{`" gate is both sufficient
/// and required: `export function/class/const ...` bodies/values
/// legitimately contain unrelated, unbalanced braces on their own opening
/// line and must NOT be swallowed by this join (a real perf regression
/// for large exported functions/components, though not a correctness
/// issue either way — see the module doc comment on `.exports`'
/// zero-consumer status).
fn needs_clause_join(trimmed: &str, is_import: bool) -> bool {
    if is_import {
        brace_depth_delta(trimmed) > 0
    } else {
        let after = trimmed
            .strip_prefix("export")
            .unwrap_or(trimmed)
            .trim_start();
        let after = after
            .strip_prefix("type ")
            .map(str::trim_start)
            .unwrap_or(after);
        after.starts_with('{')
    }
}

/// Classifies one already-joined `import ...` logical line. Mirrors
/// `determine_import_kind`'s precedence by construction: a default
/// binding, when present, always appears first in the clause text, so
/// checking the clause's leading character naturally reproduces
/// "Default wins over Namespace/Named when combined" without needing
/// tree-sitter's explicit child-kind lookups.
///
/// Specifier text comes from [`from_clause_specifier`] (non-side-effect
/// forms) or [`leading_quoted_string`] (side-effect form) rather than
/// `tree_shake::extract_specifier`'s blind trailing-quote scan — see
/// those functions' doc comments for why: a bare trailing-quote scan
/// misreads an import-attributes clause's attribute value (`from
/// './data.json' with { type: 'json' }`) as the specifier.
fn classify_import_line(trimmed: &str) -> Option<ImportDeclaration> {
    let after = trimmed.strip_prefix("import")?.trim_start();
    let after_type = after.strip_prefix("type ").map(str::trim_start);
    let clause = after_type.unwrap_or(after);

    if clause.starts_with('\'') || clause.starts_with('"') {
        let specifier = leading_quoted_string(clause)?;
        return Some(ImportDeclaration {
            source: specifier,
            kind: ImportKind::SideEffect,
        });
    }
    // No resolvable `from '...'` clause (e.g. still-unterminated after the
    // join bound, or a bizarre shape) -- nothing to feed the crawl, safely
    // skipped.
    let specifier = from_clause_specifier(trimmed)?;
    let kind = if clause.starts_with('*') {
        ImportKind::Namespace
    } else if clause.starts_with('{') {
        ImportKind::Named
    } else {
        ImportKind::Default
    };
    Some(ImportDeclaration {
        source: specifier,
        kind,
    })
}

/// Classifies one already-joined `export ...` logical line, returning its
/// `ExportDeclaration` plus, when the export has a `from '...'` source,
/// the extra `ImportDeclaration` `extract_from_node`'s `export_statement`
/// arm also pushes into `static_imports` for the same re-export (always
/// `ImportKind::Named` regardless of the export's own kind — matched
/// here verbatim, including for `export type { .. } from '...'`, whose
/// extra crawl edge tree-sitter also registers unconditionally; the
/// separate `runtime_static_imports` post-transform narrowing step
/// prunes type-only entries back out for TS files exactly as it does
/// today).
///
/// `source` comes from [`from_clause_specifier`], NOT
/// `tree_shake::extract_specifier`'s blind trailing-quote scan: an
/// ordinary value/function/class/default export's own body or value very
/// often ends in an unrelated quoted string literal (`export const
/// UNMOUNTED = 'unmounted';`, `export default 'literal';` — both real
/// shapes, the first straight out of `react-transition-group`'s
/// `Transition.js`), which a blind trailing-quote scan would misread as a
/// re-export source, fabricating a bogus `static_imports` edge the crawl
/// then fails to resolve as a package. Requiring a genuine whole-word
/// `from` immediately before the specifier (with string-literal *content*
/// opaquely skipped while searching for it — see
/// [`from_clause_specifier`]) is what tree-sitter's grammar-aware
/// `source`-field lookup gets for free and a naive scan does not.
fn classify_export_line(trimmed: &str) -> Option<(ExportDeclaration, Option<ImportDeclaration>)> {
    let after = trimmed.strip_prefix("export")?.trim_start();
    let after_type = after.strip_prefix("type ").map(str::trim_start);
    let clause = after_type.unwrap_or(after);
    let source = from_clause_specifier(trimmed);

    let kind = if clause.starts_with('*') {
        ExportKind::All
    } else if clause.starts_with('{') {
        ExportKind::Named
    } else if clause.starts_with("default") {
        ExportKind::Default
    } else {
        ExportKind::Named
    };

    let extra_import = source.clone().map(|source| ImportDeclaration {
        source,
        kind: ImportKind::Named,
    });
    Some((ExportDeclaration { kind, source }, extra_import))
}

/// Whether `trimmed` (an already-joined `export ...` logical line) is a
/// `export * as <ident> from '<spec>'` aliased namespace re-export --
/// checked by [`extract_imports_fast`]'s caller loop *before*
/// [`classify_export_line`], forcing a whole-module bail rather than a
/// classification.
///
/// Tree-sitter wraps this shape's `*` and `as <ident>` together in its own
/// `namespace_export` node, so `parse_export_statement`'s
/// `find_child_by_kind(node, "*")` (which only looks at `export_statement`'s
/// own *direct* children) never matches it, falling through past both
/// special cases to the `Named, source: None` catch-all -- the *existing*,
/// unmodified tree-sitter path has therefore always silently dropped this
/// re-export's `source` and the crawl edge that comes with it. A structural
/// scan naturally recognizes the shape (`clause.starts_with('*')` followed
/// by a whole-word `as`) and could resolve it *correctly* -- but doing so
/// would change today's established crawl behavior as a side effect of a
/// perf-only change (WI #1997 is not a correctness audit of the pre-existing
/// tree-sitter extraction), so this bails to the tree-sitter fallback
/// instead of "fixing" it.
fn is_aliased_namespace_reexport(trimmed: &str) -> bool {
    let after = match trimmed.strip_prefix("export") {
        Some(rest) => rest.trim_start(),
        None => return false,
    };
    let after_type = after.strip_prefix("type ").map(str::trim_start);
    let clause = after_type.unwrap_or(after);
    let after_star = match clause.strip_prefix('*') {
        Some(rest) => rest.trim_start(),
        None => return false,
    };
    after_star == "as"
        || matches!(
            after_star.strip_prefix("as"),
            Some(rest) if rest.starts_with(|c: char| c.is_whitespace())
        )
}

/// Extracts the module specifier from a genuine `... from '<spec>'` /
/// `... from "<spec>"` clause in an already-joined import/export logical
/// line, or `None` if the line has no `from` clause at all. Searches
/// left to right for a whole-word `from` immediately (whitespace only)
/// followed by a quote, treating every plain-string span it passes over
/// as opaque (jumping straight to that string's own matching close-quote
/// without inspecting its content) so neither an import binding literally
/// named `from` (`import { from } from './utils';` — legal, `from` is not
/// a reserved word) nor a `from '...'`-shaped substring sitting inert
/// inside an unrelated string's content can be mistaken for the real
/// clause. Returns the FIRST such match, matching JS/TS grammar's
/// guarantee that a statement has at most one `from` clause.
fn from_clause_specifier(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    while i < n {
        let b = bytes[i];
        if b == b'\'' || b == b'"' {
            let quote = b;
            let mut j = i + 1;
            while j < n {
                if bytes[j] == b'\\' && j + 1 < n {
                    j += 2;
                    continue;
                }
                if bytes[j] == quote {
                    j += 1;
                    break;
                }
                j += 1;
            }
            i = j;
            continue;
        }
        if i + 4 <= n && &bytes[i..i + 4] == b"from" {
            let boundary_before = i == 0 || !is_ident_byte(bytes[i - 1]);
            let boundary_after = i + 4 == n || !is_ident_byte(bytes[i + 4]);
            if boundary_before && boundary_after {
                let mut k = i + 4;
                while k < n && matches!(bytes[k], b' ' | b'\t') {
                    k += 1;
                }
                if k < n && (bytes[k] == b'\'' || bytes[k] == b'"') {
                    let quote = bytes[k];
                    let start = k + 1;
                    let mut m = start;
                    while m < n && bytes[m] != quote {
                        m += 1;
                    }
                    if m < n {
                        return Some(s[start..m].to_string());
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Reads the specifier a side-effect import/export clause itself opens
/// with (`clause` is already known, by the caller, to start with a quote
/// character) — stops at the first matching unescaped close-quote, so
/// anything after it (an import-attributes `with { type: '...' }` /
/// `assert { ... }` clause) is correctly ignored rather than folded into
/// the specifier.
fn leading_quoted_string(clause: &str) -> Option<String> {
    let bytes = clause.as_bytes();
    let quote = *bytes.first()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let mut j = 1usize;
    while j < bytes.len() {
        if bytes[j] == b'\\' && j + 1 < bytes.len() {
            j += 2;
            continue;
        }
        if bytes[j] == quote {
            return Some(clause[1..j].to_string());
        }
        j += 1;
    }
    None
}

/// Whole-source scan for `require(...)`/`import(...)` call-expression
/// targets in `line_view` (comments and template-literal spans already
/// blanked by [`lex_scan`]; plain-string *content* is left intact, so
/// this walk tracks quote spans itself to step over unrelated string
/// data without inspecting it — see the inline comment below).
///
/// Mirrors `tree_shake::extract_dynamic_import_targets_from`'s proven
/// shape: a fixed `require(`/`import(` needle, a leading
/// identifier-boundary disqualifier (rules out `myrequire(`/`reimport(`;
/// the fixed needle already rules out a trailing one, e.g. `requireFoo(`
/// does not contain the literal substring `require(`), and a
/// literal-quote-only specifier (a template literal or identifier
/// argument is silently skipped, matching `is_dynamic_import`/
/// `extract_dynamic_import` and CJS `require(` handling in
/// `extract_from_node`). Unlike `scan_require_call` (which lacks the
/// leading-boundary check and is comment/string-blind — an accepted
/// low-stakes trade-off for its own tree-shaking "non-resolution is a
/// safe net" use), both checks are load-bearing here: an unresolved
/// specifier during crawl is a build warning/failure, not a silent no-op.
fn scan_calls(line_view: &str) -> (Vec<(usize, ImportDeclaration)>, Vec<(usize, String)>) {
    let bytes = line_view.as_bytes();
    let n = bytes.len();
    let mut requires: Vec<(usize, ImportDeclaration)> = Vec::new();
    let mut dynamics: Vec<(usize, String)> = Vec::new();
    let mut i = 0usize;
    while i < n {
        let b = bytes[i];
        if b == b'\'' || b == b'"' {
            // Step over a plain string's content without inspecting it --
            // `lex_scan` already proved every such span in `line_view` is
            // well-formed, so this walk can assume that same shape.
            let quote = b;
            let mut j = i + 1;
            while j < n {
                if bytes[j] == b'\\' && j + 1 < n {
                    j += 2;
                    continue;
                }
                if bytes[j] == quote {
                    j += 1;
                    break;
                }
                j += 1;
            }
            i = j;
            continue;
        }
        let (needle, is_require): (&[u8], bool) = if bytes[i..].starts_with(b"require(") {
            (b"require(".as_slice(), true)
        } else if bytes[i..].starts_with(b"import(") {
            (b"import(".as_slice(), false)
        } else {
            i += 1;
            continue;
        };
        let word_start = i;
        if word_start > 0 && is_ident_byte(bytes[word_start - 1]) {
            i = word_start + 1;
            continue;
        }
        let after_paren = word_start + needle.len();
        let mut k = after_paren;
        while k < n && matches!(bytes[k], b' ' | b'\t' | b'\n') {
            k += 1;
        }
        if k >= n || (bytes[k] != b'\'' && bytes[k] != b'"') {
            i = after_paren;
            continue;
        }
        let quote = bytes[k];
        let spec_start = k + 1;
        let mut m = spec_start;
        let mut closed = false;
        while m < n {
            if bytes[m] == b'\\' && m + 1 < n {
                m += 2;
                continue;
            }
            if bytes[m] == quote {
                closed = true;
                break;
            }
            if bytes[m] == b'\n' {
                break;
            }
            m += 1;
        }
        if !closed {
            i = after_paren;
            continue;
        }
        let specifier = &line_view[spec_start..m];
        if is_require {
            if specifier.contains(".development.") || specifier.contains("-development.") {
                tracing::debug!("Skipping development build: {}", specifier);
            } else {
                requires.push((
                    word_start,
                    ImportDeclaration {
                        source: specifier.to_string(),
                        kind: ImportKind::Default,
                    },
                ));
            }
        } else {
            dynamics.push((word_start, specifier.to_string()));
        }
        i = m + 1;
    }
    (requires, dynamics)
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

// ─── Alias resolution helper ─────────────────────────────────────────────────

/// Apply module path alias mappings to a specifier string.
///
/// Called during module graph construction (before the Node.js resolver) so
/// that alias-based imports like `"@/components/Foo"` are normalised to
/// `"./src/components/Foo"` before any `node_modules` lookup.
///
/// `aliases` is a slice of `(prefix, replacement_path_str)` pairs sorted by
/// descending prefix length so longer prefixes win.  For example:
///
/// ```text
/// [("@/", "./src/")]
/// ```
///
/// Given specifier `"@/components/Foo"`, returns `"./src/components/Foo"`.
/// If no prefix matches the specifier is returned unchanged.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn apply_alias(specifier: &str, aliases: &[(String, String)]) -> String {
    for (prefix, replacement) in aliases {
        if specifier.starts_with(prefix.as_str()) {
            let rest = &specifier[prefix.len()..];
            return format!("{}{}", replacement, rest);
        }
    }
    specifier.to_string()
}

// ─── SVGR routing (import `.svg` as a React component) ────────────────────────

/// True when `specifier` points at a `.svg` file (case-insensitive, query
/// strings like `?url` / `?react` stripped).
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_svg_specifier(specifier: &str) -> bool {
    let path = specifier.split(['?', '#']).next().unwrap_or(specifier);
    path.to_ascii_lowercase().ends_with(".svg")
}

// ─── SCSS / Sass routing (compile `.scss`/`.sass` imports to CSS) ─────────────

/// True when `specifier` points at a `.scss`/`.sass` Sass source
/// (case-insensitive, query strings stripped). These imports must be routed
/// through the grass SCSS compile step ([`crate::css::scss`]) before the
/// lightningcss pipeline, rather than read as plain CSS.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_scss_specifier(specifier: &str) -> bool {
    let path = specifier.split(['?', '#']).next().unwrap_or(specifier);
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".scss") || lower.ends_with(".sass")
}

/// Classify a style import specifier into the route the build must take.
///
/// - `.scss`/`.sass` → [`StyleImportRoute::Sass`] (compile via grass first).
/// - any other style specifier (`.css`, `.less`, …) → [`StyleImportRoute::PlainCss`].
///
/// Routing strictly by extension keeps plain `.css` on the existing path and
/// only diverts the Sass family through the compile step.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn classify_style_import(specifier: &str) -> StyleImportRoute {
    if is_scss_specifier(specifier) {
        StyleImportRoute::Sass
    } else {
        StyleImportRoute::PlainCss
    }
}

/// How a style import is fed into the CSS pipeline.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleImportRoute {
    /// `.scss`/`.sass` — compile to CSS via grass before lightningcss.
    Sass,
    /// `.css` (and other already-CSS forms) — pass straight through.
    PlainCss,
}

/// Decide whether a `.svg` import should be routed through SVGR (emit a React
/// component module) instead of the default asset-URL behavior.
///
/// This mirrors `vite-plugin-svgr`'s routing, which is driven by two things:
///
/// 1. **Explicit query suffix** — `?url` forces the asset-URL path even when
///    SVGR is enabled; `?react` (or `?component`) forces the component path
///    even when SVGR is disabled globally.
/// 2. **The import shape** — with `vite-plugin-svgr`'s `{ exportType: 'named'
///    }` (what `fe-shared` uses) the SVG is a component only when imported via
///    the named `ReactComponent` binding: `import { ReactComponent as Icon }
///    from './icon.svg'`. A bare `import url from './icon.svg'` stays an asset
///    URL. With `exportType: 'default'`, the default import is the component.
///
/// `svgr_enabled` is the global toggle ([`crate::asset::SvgrConfig::enabled`]);
/// `export_type` is the configured [`crate::asset::SvgrExportType`].
///
/// Returns `true` to route through `transform_svg_to_component`, `false` to
/// keep the existing asset-URL emission. Non-`.svg` specifiers always return
/// `false`.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn should_route_svg_as_component(
    specifier: &str,
    import_kind: &ImportKind,
    svgr_enabled: bool,
    export_type: crate::asset::SvgrExportType,
) -> bool {
    if !is_svg_specifier(specifier) {
        return false;
    }

    // 1. Explicit query suffix wins over the global toggle.
    let query = specifier.split('?').nth(1).unwrap_or("");
    if query.contains("url") {
        // `import logo from './logo.svg?url'` — always an asset URL.
        return false;
    }
    if query.contains("react") || query.contains("component") {
        // `import Logo from './logo.svg?react'` — always a component.
        return true;
    }

    if !svgr_enabled {
        return false;
    }

    // 2. Import-shape gate, matching the configured export type.
    use crate::asset::SvgrExportType;
    match export_type {
        // Named (`{ exportType: 'named' }`, fe-shared default): only the named
        // `ReactComponent` binding is the component. A default/namespace/
        // side-effect import keeps the asset-URL behavior.
        SvgrExportType::Named => matches!(import_kind, ImportKind::Named),
        // Default: the default import is the component.
        SvgrExportType::Default => matches!(import_kind, ImportKind::Default),
        // Both: either a named or default import resolves to the component.
        SvgrExportType::Both => {
            matches!(import_kind, ImportKind::Named | ImportKind::Default)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_static_imports() {
        let source = r#"
            import React from 'react';
            import { useState } from 'react';
            import * as utils from './utils';
            import './styles.css';
        "#;

        let imports = extract_imports(source, false).unwrap();

        assert_eq!(imports.static_imports.len(), 4);
        assert_eq!(imports.static_imports[0].source, "react");
        assert_eq!(imports.static_imports[0].kind, ImportKind::Default);
        assert_eq!(imports.static_imports[1].source, "react");
        assert_eq!(imports.static_imports[1].kind, ImportKind::Named);
        assert_eq!(imports.static_imports[2].source, "./utils");
        assert_eq!(imports.static_imports[2].kind, ImportKind::Namespace);
        assert_eq!(imports.static_imports[3].source, "./styles.css");
        assert_eq!(imports.static_imports[3].kind, ImportKind::SideEffect);
    }

    #[test]
    fn test_extract_dynamic_imports() {
        let source = r#"
            const module = import('./dynamic-module');
            async function load() {
                const mod = await import('./lazy-module');
            }
        "#;

        let imports = extract_imports(source, false).unwrap();

        assert_eq!(imports.dynamic_imports.len(), 2);
        assert_eq!(imports.dynamic_imports[0], "./dynamic-module");
        assert_eq!(imports.dynamic_imports[1], "./lazy-module");
    }

    #[test]
    fn test_extract_typescript_imports() {
        let source = r#"
            import type { User } from './types';
            import React from 'react';
        "#;

        let imports = extract_imports(source, true).unwrap();

        assert!(imports.static_imports.len() >= 1);
        assert!(imports.static_imports.iter().any(|i| i.source == "react"));
    }

    #[test]
    fn test_extract_exports() {
        let source = r#"
            export const foo = 1;
            export default function bar() {}
            export * from './other';
        "#;

        let imports = extract_imports(source, false).unwrap();

        assert_eq!(imports.exports.len(), 3);
    }

    // ─── Alias integration tests ──────────────────────────────────────────────

    /// REQ-JET-05/REQ-JET-07: apply_alias correctly maps aliased specifiers to
    /// their target paths — the same function used in both dev (JIT) and prod
    /// (bundler) module graph construction.
    #[test]
    fn alias_works_in_prod_build() {
        let aliases = vec![("@/".to_string(), "./src/".to_string())];

        // Aliased import resolves correctly
        assert_eq!(
            apply_alias("@/components/Foo", &aliases),
            "./src/components/Foo"
        );
        assert_eq!(
            apply_alias("@/utils/helpers", &aliases),
            "./src/utils/helpers"
        );

        // Non-aliased specifiers are returned unchanged
        assert_eq!(apply_alias("react", &aliases), "react");
        assert_eq!(apply_alias("./local-module", &aliases), "./local-module");
        assert_eq!(
            apply_alias("../parent-module", &aliases),
            "../parent-module"
        );
    }

    /// REQ-JET-06: longest alias prefix wins when multiple entries are defined.
    #[test]
    fn alias_longest_prefix_wins() {
        let aliases = vec![
            // Sorted longest-first (as AliasResolver produces)
            ("@/components/".to_string(), "./src/ui/".to_string()),
            ("@/".to_string(), "./src/".to_string()),
        ];

        // More specific prefix wins
        assert_eq!(
            apply_alias("@/components/Button", &aliases),
            "./src/ui/Button"
        );

        // Less specific prefix used when no longer match
        assert_eq!(
            apply_alias("@/hooks/useData", &aliases),
            "./src/hooks/useData"
        );
    }

    /// REQ-JET-07: apply_alias is deterministic across calls (prod == dev).
    #[test]
    fn alias_resolution_is_deterministic() {
        let aliases = vec![("@/".to_string(), "./src/".to_string())];
        let specifier = "@/pages/Home";

        // Calling apply_alias multiple times always yields the same result
        let result_1 = apply_alias(specifier, &aliases);
        let result_2 = apply_alias(specifier, &aliases);
        assert_eq!(result_1, result_2);
    }

    // ─── SVGR routing tests ───────────────────────────────────────────────────

    use crate::asset::SvgrExportType;

    #[test]
    fn is_svg_specifier_detects_svg() {
        assert!(is_svg_specifier("./icon.svg"));
        assert!(is_svg_specifier("./Icon.SVG"));
        assert!(is_svg_specifier("./icon.svg?react"));
        assert!(is_svg_specifier("@/assets/logo.svg?url"));
        assert!(!is_svg_specifier("./icon.png"));
        assert!(!is_svg_specifier("react"));
    }

    #[test]
    fn named_export_routes_only_reactcomponent_named_import() {
        // fe-shared shape: `import { ReactComponent as Icon } from './icon.svg'`
        assert!(should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Named,
            true,
            SvgrExportType::Named,
        ));
        // Bare default import stays an asset URL under `exportType: 'named'`.
        assert!(!should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Default,
            true,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn default_export_routes_default_import() {
        assert!(should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Default,
            true,
            SvgrExportType::Default,
        ));
        assert!(!should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Named,
            true,
            SvgrExportType::Default,
        ));
    }

    #[test]
    fn url_query_forces_asset_url_even_when_enabled() {
        assert!(!should_route_svg_as_component(
            "./icon.svg?url",
            &ImportKind::Named,
            true,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn react_query_forces_component_even_when_disabled() {
        assert!(should_route_svg_as_component(
            "./icon.svg?react",
            &ImportKind::Default,
            false,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn disabled_svgr_keeps_asset_url() {
        assert!(!should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Named,
            false,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn non_svg_never_routes_as_component() {
        assert!(!should_route_svg_as_component(
            "./icon.png",
            &ImportKind::Named,
            true,
            SvgrExportType::Named,
        ));
    }

    // ─── SCSS / Sass routing tests ─────────────────────────────────────────────

    #[test]
    fn is_scss_specifier_detects_scss_and_sass() {
        assert!(is_scss_specifier("./theme.scss"));
        assert!(is_scss_specifier("./theme.SCSS"));
        assert!(is_scss_specifier("../styles/main.sass"));
        assert!(is_scss_specifier("@/styles/x.scss?inline"));
        assert!(!is_scss_specifier("./theme.css"));
        assert!(!is_scss_specifier("./icon.svg"));
        assert!(!is_scss_specifier("react"));
    }

    #[test]
    fn classify_routes_sass_family_and_keeps_plain_css() {
        assert_eq!(classify_style_import("./a.scss"), StyleImportRoute::Sass);
        assert_eq!(classify_style_import("./a.sass"), StyleImportRoute::Sass);
        // Plain CSS (and other forms) must NOT be diverted.
        assert_eq!(classify_style_import("./a.css"), StyleImportRoute::PlainCss);
        assert_eq!(
            classify_style_import("./a.less"),
            StyleImportRoute::PlainCss
        );
    }

    // ─── Fast-path differential harness (#1997) ────────────────────────────────
    //
    // `extract_imports_fast`'s trust anchor: every fixture below must produce
    // byte-for-byte the same `ModuleImports` the tree-sitter walk
    // (`extract_imports_with_tree`) produces, whenever the fast path claims
    // eligibility (returns `Some`, i.e. does not bail). A plain-JS-shaped
    // fixture is checked against both grammars (`is_typescript` true and
    // false) since the fast scan itself never distinguishes them; a TS-only
    // fixture (`import type`, ...) is checked against the TSX grammar only,
    // since parsing TS-only syntax under the plain JS grammar is not a
    // meaningful comparison.

    /// Panics with the fixture source on any divergence, or if the fast path
    /// unexpectedly bails (every fixture fed to this helper is chosen to be
    /// eligible).
    fn assert_fast_matches_tree(source: &str, is_typescript: bool) {
        let fast = extract_imports_fast(source).unwrap_or_else(|| {
            panic!("fast path unexpectedly bailed (is_typescript={is_typescript}) on:\n{source}")
        });
        let (tree_walk, _tree) =
            extract_imports_with_tree(source, is_typescript).unwrap_or_else(|e| {
                panic!("tree-sitter walk failed (is_typescript={is_typescript}): {e}")
            });
        assert_eq!(
            fast, tree_walk,
            "fast path diverged from tree-sitter walk (is_typescript={is_typescript}) for:\n{source}"
        );
    }

    /// Plain-JS-shaped fixture: valid under both grammars, must agree with
    /// both (the fast path itself never distinguishes them).
    fn assert_fast_matches_tree_both_grammars(source: &str) {
        assert_fast_matches_tree(source, false);
        assert_fast_matches_tree(source, true);
    }

    #[test]
    fn fast_path_matches_tree_walk_for_static_import_kinds() {
        for source in [
            "import React from 'react';",
            "import { useState, useEffect } from 'react';",
            "import * as utils from './utils';",
            "import './styles.css';",
            "import React, { useState } from 'react';",
            "import Foo from \"./foo\";",
            "import Foo from './foo'; // trailing comment",
            "// leading comment\nimport Foo from './foo';",
            // `from` used as a named binding, not the keyword -- legal JS
            // (`from` is not a reserved word); the specifier scan must skip
            // past it and find the real `from` clause.
            "import { from } from './utils';",
        ] {
            assert_fast_matches_tree_both_grammars(source);
        }
    }

    #[test]
    fn fast_path_matches_tree_walk_for_multiline_wrapped_import() {
        let source = "import {\n  Foo,\n  Bar,\n  Baz,\n} from './multi';\n";
        assert_fast_matches_tree_both_grammars(source);
    }

    #[test]
    fn fast_path_matches_tree_walk_for_multiline_wrapped_export() {
        let source = "export {\n  Foo,\n  Bar,\n} from './multi';\n";
        assert_fast_matches_tree_both_grammars(source);
    }

    #[test]
    fn fast_path_matches_tree_walk_for_dynamic_imports_and_requires() {
        for source in [
            "const mod = import('./dynamic-module');",
            "async function load() {\n  const mod = await import('./lazy-module');\n}",
            "const b = import(\"./dynamic-b\");",
            "const foo = require('./foo');",
            "const { bar } = require('bar-pkg');",
            // Non-top-level: dynamic import/require nested inside a
            // function/conditional body -- `scan_calls` walks the whole
            // `line_view`, matching tree-sitter's unrestricted-depth
            // `call_expression` walk.
            "function load() {\n  if (cond) {\n    return require('./nested');\n  }\n}",
            // Non-literal argument: silently skipped by both, not a bail.
            "const mod = import(moduleName);",
            "const mod = require(moduleName);",
        ] {
            assert_fast_matches_tree_both_grammars(source);
        }
    }

    #[test]
    fn fast_path_matches_tree_walk_for_import_meta() {
        // `import.meta.*` must not be mistaken for an import statement or a
        // dynamic-import call by either scanner.
        let source = "const dev = import.meta.env.DEV;\nimport React from 'react';";
        assert_fast_matches_tree_both_grammars(source);
    }

    #[test]
    fn fast_path_matches_tree_walk_for_export_kinds() {
        for source in [
            "export const foo = 1;",
            "export default function bar() {}",
            "export default class Foo {}",
            "export * from './other';",
            "export { foo, bar };",
            "export { foo as default };",
            "export { Foo } from './foo';",
            // Regression (#1997): a plain value export whose own string
            // value must NOT be misread as a re-export source -- the exact
            // line from react-transition-group's Transition.js that
            // fabricated a bogus `static_imports` edge before the
            // `from_clause_specifier` rewrite.
            "export const UNMOUNTED = 'unmounted';",
        ] {
            assert_fast_matches_tree_both_grammars(source);
        }
    }

    #[test]
    fn fast_path_matches_tree_walk_for_typescript_only_constructs() {
        for source in [
            "import type { User } from './types';",
            "import type Foo from './foo';",
            "export type { Foo } from './types';",
        ] {
            assert_fast_matches_tree(source, true);
        }
    }

    #[test]
    fn fast_path_matches_tree_walk_for_import_attributes() {
        // Regression (#1997): import-attributes clauses (`with { ... }`, or
        // the older `assert { ... }` form) must not contaminate the
        // specifier -- a blind trailing-quote scan would misread the
        // attribute value ('json') as the module source.
        for source in [
            "import data from './data.json' with { type: 'json' };",
            "import data from './data.json' assert { type: 'json' };",
        ] {
            assert_fast_matches_tree(source, true);
        }
    }

    #[test]
    fn fast_path_matches_tree_walk_with_unrelated_template_literal_present() {
        // A template literal elsewhere in the file (including one with a
        // `${}` substitution) must not confuse the statement scanner that
        // runs on `lex_scan`'s blanked-out view.
        let source =
            "const x = `hello ${name}`;\nimport React from 'react';\nexport const y = `world`;";
        assert_fast_matches_tree_both_grammars(source);
    }

    // ─── Eligibility-guard bail cases (#1997) ──────────────────────────────────
    //
    // Each of these must make `extract_imports_fast` return `None` (a bail
    // to the tree-sitter fallback) -- never a wrong answer.

    #[test]
    fn fast_path_bails_on_unterminated_string() {
        let source = "const x = 'unterminated;\nimport React from 'react';";
        assert!(extract_imports_fast(source).is_none());
    }

    #[test]
    fn fast_path_bails_on_unterminated_block_comment() {
        let source = "/* not closed\nimport React from 'react';";
        assert!(extract_imports_fast(source).is_none());
    }

    #[test]
    fn fast_path_bails_on_nested_template_literal() {
        let source = "const x = `outer ${`inner`}`;\nimport React from 'react';";
        assert!(extract_imports_fast(source).is_none());
    }

    #[test]
    fn fast_path_bails_on_string_spanning_a_raw_newline() {
        let source = "const x = 'line1\nline2';\nimport React from 'react';";
        assert!(extract_imports_fast(source).is_none());
    }

    #[test]
    fn fast_path_bails_on_overlong_line() {
        let filler = "a".repeat(4001);
        let source = format!("const x = 1; // {filler}\nimport React from 'react';");
        assert!(extract_imports_fast(&source).is_none());
    }

    #[test]
    fn fast_path_bails_on_aliased_namespace_reexport() {
        // `export * as ns from '...'`: tree-sitter wraps `*`+`as ns` in a
        // `namespace_export` node, so `parse_export_statement`'s
        // direct-child `find_child_by_kind(node, "*")` never matches --
        // the *existing*, unmodified tree-sitter path has always silently
        // dropped this re-export's `source` (falls through to `Named,
        // None`), meaning the crawl has never followed this edge. The
        // fast path must bail here (not resolve the edge "correctly"),
        // or it would change today's established crawl behavior as a
        // side effect of a perf-only change -- see
        // `is_aliased_namespace_reexport`'s doc comment.
        let source = "export * as ns from './namespace';";
        assert!(extract_imports_fast(source).is_none());
        // Sanity: confirm the tree-sitter side's gap this bail is
        // preserving actually still exists (so this test would fail loudly
        // if a tree-sitter upgrade ever fixes it upstream, signaling the
        // bail rule can be revisited).
        let (tree_walk, _tree) = extract_imports_with_tree(source, false).unwrap();
        assert_eq!(
            tree_walk.exports,
            vec![ExportDeclaration {
                kind: ExportKind::Named,
                source: None,
            }]
        );
        assert!(tree_walk.static_imports.is_empty());
    }
}
// CODEGEN-END
