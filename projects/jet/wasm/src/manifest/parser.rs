// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
// CODEGEN-BEGIN
//! `jet.declare.d.ts` parser + nearest-ancestor overlay-merge loader.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/binding-manifest.md
//!
//! Implements the manifest grammar (G1–G5 / G-reject-1..5) using a
//! line-and-brace hand-rolled parser. tree-sitter-typescript would be
//! cleaner but adds a heavy dep that the v0 grammar does not justify;
//! upgrading to tree-sitter is a follow-up once jet-tsx-to-rust lands
//! and brings the parser in as a runtime dep.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::defaults::DEFAULT_BINDINGS;

/// Top-level shape returned by [`parse_manifest`].
///
/// The `entries` list is the result of overlay-merging every
/// `jet.declare.d.ts` found between the source file and the workspace
/// root, with inner manifests winning on per-`module_name` conflict.
/// The default bindings ([`DEFAULT_BINDINGS`]) are seeded first and
/// can be overridden by user-supplied manifests.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedManifest {
    pub entries: Vec<ModuleEntry>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleEntry {
    pub module_name: String,
    pub exports: Vec<ExportEntry>,
    pub jet_impl: JetImpl,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ExportEntry {
    pub kind: ExportKind,
    pub name: String,
    pub signature: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Default,
    Named,
}

/// Discriminated union for `__jet_impl`. Either `Rust` (reimplemented in
/// Rust) or `Bridge` (wasm-bindgen bridge surface).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum JetImpl {
    Rust { symbol: String },
    Bridge { symbol: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ManifestError {
    pub code: ManifestErrorCode,
    pub message: String,
    pub path: Option<PathBuf>,
    pub line: Option<usize>,
    pub col: Option<usize>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestErrorCode {
    /// `MANIFEST_PARSE_001` — `jet.declare.d.ts` path does not exist.
    FileNotFound,
    /// `MANIFEST_PARSE_002` — file is not valid TypeScript ambient syntax.
    ParseError,
    /// `MANIFEST_PARSE_003` — `declare module` block has empty/absent string literal name.
    MissingModuleName,
    /// `MANIFEST_PARSE_004` — `__jet_impl` value does not start with `rust:` or `bridge:`.
    UnknownImplDiscriminant,
    /// `MANIFEST_PARSE_005` — duplicate module entries after overlay merge.
    DuplicateModule,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
impl ManifestErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileNotFound => "MANIFEST_PARSE_001",
            Self::ParseError => "MANIFEST_PARSE_002",
            Self::MissingModuleName => "MANIFEST_PARSE_003",
            Self::UnknownImplDiscriminant => "MANIFEST_PARSE_004",
            Self::DuplicateModule => "MANIFEST_PARSE_005",
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
impl std::error::Error for ManifestError {}

// ── Public API ─────────────────────────────────────────────────────

/// Walk ancestors of `source_dir` up to `workspace_root` collecting
/// every `jet.declare.d.ts` encountered, then overlay-merge from the
/// workspace root down to the source dir. Inner manifests win on
/// per-`module_name` conflict. Default bindings are seeded first.
///
/// Spec: `binding-manifest.md` § Manifest Discovery Logic.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
pub fn parse_manifest(
    source_dir: &Path,
    workspace_root: &Path,
) -> Result<ParsedManifest, ManifestError> {
    let chain = ancestor_chain(source_dir, workspace_root);
    let mut merged: HashMap<String, ModuleEntry> = HashMap::new();
    // Seed defaults first; user manifests override.
    for entry in DEFAULT_BINDINGS.iter().cloned() {
        merged.insert(entry.module_name.clone(), entry);
    }
    // Walk root → source_dir so inner overrides outer.
    for dir in chain.iter().rev() {
        let path = dir.join("jet.declare.d.ts");
        if !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| ManifestError {
            code: ManifestErrorCode::FileNotFound,
            message: format!("could not read manifest at {}: {e}", path.display()),
            path: Some(path.clone()),
            line: None,
            col: None,
        })?;
        let entries = parse_manifest_text(&text).map_err(|mut e| {
            e.path = Some(path.clone());
            e
        })?;
        for entry in entries {
            merged.insert(entry.module_name.clone(), entry);
        }
    }
    let mut entries: Vec<ModuleEntry> = merged.into_values().collect();
    entries.sort_by(|a, b| a.module_name.cmp(&b.module_name));
    Ok(ParsedManifest { entries })
}

/// Parse a single manifest file's text into the list of module
/// entries it declares. Does not seed defaults or overlay-merge.
/// Used directly in tests and by [`parse_manifest`] per file.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
pub fn parse_manifest_text(text: &str) -> Result<Vec<ModuleEntry>, ManifestError> {
    let mut out: Vec<ModuleEntry> = Vec::new();
    let mut seen_names: HashMap<String, usize> = HashMap::new();
    let mut iter = text.lines().enumerate().peekable();

    while let Some((line_no, raw)) = iter.next() {
        let line = raw.trim();
        // Skip blank + line comments.
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        // Block-comment lines we don't fully tokenize: a leading `/*`
        // or `*` is treated as comment. /** … */ on a single line is
        // skipped; multi-line block comments must end with `*/` on
        // their own line.
        if line.starts_with("/*") || line.starts_with('*') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("declare module") {
            let (name, tail, line_pos) = parse_module_name(rest, line_no + 1)?;
            // Consume body until matching `}`. Pass the tail of the
            // current line so an inline `{` is honored.
            let body = collect_module_body(&mut iter, line_no + 1, tail.as_str())?;
            let entry = parse_module_body(&name, &body, line_pos)?;
            if let Some(prev) = seen_names.insert(entry.module_name.clone(), line_pos) {
                let _ = prev;
                return Err(ManifestError {
                    code: ManifestErrorCode::DuplicateModule,
                    message: format!(
                        "duplicate module \"{}\" in same file (line {})",
                        entry.module_name, line_pos
                    ),
                    path: None,
                    line: Some(line_pos),
                    col: None,
                });
            }
            out.push(entry);
            continue;
        }

        // Anything else at the top level is a non-ambient statement
        // (G-reject-5) or an unrecognized construct.
        return Err(ManifestError {
            code: ManifestErrorCode::ParseError,
            message: format!(
                "expected `declare module \"...\" {{ ... }}` at top level, got: {} (line {})",
                truncate(line, 60),
                line_no + 1
            ),
            path: None,
            line: Some(line_no + 1),
            col: None,
        });
    }

    Ok(out)
}

// ── Internal: line-and-brace parser ────────────────────────────────

fn parse_module_name(
    rest_after_keyword: &str,
    line: usize,
) -> Result<(String, String, usize), ManifestError> {
    let s = rest_after_keyword.trim_start();
    let (quoted, tail) = take_string_literal(s).ok_or(ManifestError {
        code: ManifestErrorCode::MissingModuleName,
        message: format!("expected \"<module-name>\" after `declare module` (line {line})"),
        path: None,
        line: Some(line),
        col: None,
    })?;
    if quoted.is_empty() {
        return Err(ManifestError {
            code: ManifestErrorCode::MissingModuleName,
            message: format!("empty module name in `declare module` (line {line})"),
            path: None,
            line: Some(line),
            col: None,
        });
    }
    Ok((quoted, tail.to_string(), line))
}

fn take_string_literal(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start();
    let mut chars = s.char_indices();
    let (start, q) = chars.next()?;
    if q != '"' {
        return None;
    }
    let _ = start;
    let mut end_idx = None;
    let mut prev = '"';
    for (i, c) in chars {
        if c == '"' && prev != '\\' {
            end_idx = Some(i);
            break;
        }
        prev = c;
    }
    let end = end_idx?;
    let content = &s[1..end];
    let tail = &s[end + 1..];
    Some((content.to_string(), tail))
}

fn collect_module_body<'a, I>(
    iter: &mut std::iter::Peekable<I>,
    open_line: usize,
    initial_tail: &str,
) -> Result<Vec<(usize, String)>, ManifestError>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    let mut body: Vec<(usize, String)> = Vec::new();
    let mut depth: i32 = 0;

    // The caller has consumed the `declare module "..."` line. The tail
    // is whatever followed the closing quote on that same line — it may
    // contain the opening `{` (the common single-line form
    // `declare module "x" {`), the entire body, or nothing at all.
    let initial = initial_tail.trim();
    if !initial.is_empty() {
        if let Some((_, after)) = initial.split_once('{') {
            depth = 1;
            let inline_closes = after.matches('}').count() as i32;
            depth -= inline_closes;
            let inner = after.split('}').next().unwrap_or("").trim();
            if !inner.is_empty() {
                body.push((open_line, inner.to_string()));
            }
            if depth == 0 {
                return Ok(body);
            }
        }
    }

    let mut started = depth > 0;
    for (line_no, raw) in iter.by_ref() {
        let trimmed = raw.trim();
        if !started {
            if let Some((_, after)) = trimmed.split_once('{') {
                started = true;
                depth = 1;
                let after = after.trim();
                let inline_closes = after.matches('}').count() as i32;
                depth -= inline_closes;
                let inner = after.split('}').next().unwrap_or("").trim();
                if !inner.is_empty() {
                    body.push((line_no + 1, inner.to_string()));
                }
                if depth == 0 {
                    return Ok(body);
                }
                continue;
            }
            return Err(ManifestError {
                code: ManifestErrorCode::ParseError,
                message: format!(
                    "expected `{{` after `declare module \"...\"` (line {})",
                    line_no + 1
                ),
                path: None,
                line: Some(line_no + 1),
                col: None,
            });
        }
        // Track brace depth.
        depth += trimmed.matches('{').count() as i32;
        depth -= trimmed.matches('}').count() as i32;
        if depth <= 0 {
            // Strip trailing `}` from the last line.
            let inner = trimmed.trim_end_matches('}').trim();
            if !inner.is_empty() {
                body.push((line_no + 1, inner.to_string()));
            }
            return Ok(body);
        }
        body.push((line_no + 1, trimmed.to_string()));
    }
    Err(ManifestError {
        code: ManifestErrorCode::ParseError,
        message: format!("unterminated `declare module` block (opened at line {open_line})"),
        path: None,
        line: Some(open_line),
        col: None,
    })
}

fn parse_module_body(
    name: &str,
    body: &[(usize, String)],
    open_line: usize,
) -> Result<ModuleEntry, ManifestError> {
    let mut exports: Vec<ExportEntry> = Vec::new();
    let mut jet_impl: Option<JetImpl> = None;
    for (line_no, raw) in body {
        let line = raw.trim().trim_end_matches(';').trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
            continue;
        }

        // G5 — the __jet_impl sentinel.
        if let Some(rest) = line.strip_prefix("declare const __jet_impl") {
            let after = rest.trim_start_matches(':').trim();
            let (value, _) = take_string_literal(after).ok_or(ManifestError {
                code: ManifestErrorCode::ParseError,
                message: format!("expected string literal for __jet_impl (line {line_no})"),
                path: None,
                line: Some(*line_no),
                col: None,
            })?;
            let parsed = parse_jet_impl(&value, name, *line_no)?;
            jet_impl = Some(parsed);
            continue;
        }

        // Other `declare const` are silently ignored per G5 description.
        if line.starts_with("declare const") {
            continue;
        }

        // G2 — export default function.
        if let Some(rest) = line.strip_prefix("export default function") {
            let signature = rest
                .trim_start_matches(|c: char| c.is_whitespace())
                .to_string();
            exports.push(ExportEntry {
                kind: ExportKind::Default,
                name: "default".to_string(),
                signature: Some(strip_trailing_semicolon(&signature)),
            });
            continue;
        }

        // G-reject-1 — export default arrow / class / other.
        if let Some(rest) = line.strip_prefix("export default") {
            let kind = rest.trim();
            let label = if kind.starts_with("class") {
                "class"
            } else {
                "arrow"
            };
            return Err(ManifestError {
                code: ManifestErrorCode::ParseError,
                message: format!(
                    "G-reject (export default {label}): only `export default function` is accepted (line {line_no})"
                ),
                path: None,
                line: Some(*line_no),
                col: None,
            });
        }

        // G3 — export function <name>(...).
        if let Some(rest) = line.strip_prefix("export function") {
            let trimmed = rest.trim_start();
            let (ident, after) = take_identifier(trimmed);
            if ident.is_empty() {
                return Err(ManifestError {
                    code: ManifestErrorCode::ParseError,
                    message: format!(
                        "expected function name after `export function` (line {line_no})"
                    ),
                    path: None,
                    line: Some(*line_no),
                    col: None,
                });
            }
            exports.push(ExportEntry {
                kind: ExportKind::Named,
                name: ident,
                signature: Some(strip_trailing_semicolon(after.trim())),
            });
            continue;
        }

        // G4 — export const <name>: <type>.
        if let Some(rest) = line.strip_prefix("export const") {
            let trimmed = rest.trim_start();
            let (ident, _after) = take_identifier(trimmed);
            if ident.is_empty() {
                return Err(ManifestError {
                    code: ManifestErrorCode::ParseError,
                    message: format!("expected identifier after `export const` (line {line_no})"),
                    path: None,
                    line: Some(*line_no),
                    col: None,
                });
            }
            exports.push(ExportEntry {
                kind: ExportKind::Named,
                name: ident,
                signature: None,
            });
            continue;
        }

        // G-reject-2 — import statements.
        if line.starts_with("import ") || line.starts_with("import{") {
            return Err(ManifestError {
                code: ManifestErrorCode::ParseError,
                message: format!(
                    "G-reject-2 (import statement): jet.declare.d.ts does not permit imports (line {line_no})"
                ),
                path: None,
                line: Some(*line_no),
                col: None,
            });
        }

        // G-reject-4 — nested `declare module`.
        if line.starts_with("declare module") {
            return Err(ManifestError {
                code: ManifestErrorCode::ParseError,
                message: format!(
                    "G-reject-4 (nested declare module): module declarations must be top-level (line {line_no})"
                ),
                path: None,
                line: Some(*line_no),
                col: None,
            });
        }

        return Err(ManifestError {
            code: ManifestErrorCode::ParseError,
            message: format!(
                "unrecognized declaration inside module \"{name}\": {} (line {line_no})",
                truncate(line, 60)
            ),
            path: None,
            line: Some(*line_no),
            col: None,
        });
    }

    let jet_impl = jet_impl.ok_or(ManifestError {
        code: ManifestErrorCode::ParseError,
        message: format!(
            "missing `declare const __jet_impl: ...` in module \"{name}\" (block opened at line {open_line})"
        ),
        path: None,
        line: Some(open_line),
        col: None,
    })?;

    Ok(ModuleEntry {
        module_name: name.to_string(),
        exports,
        jet_impl,
    })
}

fn parse_jet_impl(value: &str, module_name: &str, line: usize) -> Result<JetImpl, ManifestError> {
    let (disc, sym) = value.split_once(':').ok_or(ManifestError {
        code: ManifestErrorCode::UnknownImplDiscriminant,
        message: format!(
            "__jet_impl missing discriminant separator in module \"{module_name}\": {value:?} (line {line})"
        ),
        path: None,
        line: Some(line),
        col: None,
    })?;
    let symbol = sym.trim().to_string();
    if symbol.is_empty() {
        return Err(ManifestError {
            code: ManifestErrorCode::ParseError,
            message: format!(
                "__jet_impl symbol is empty in module \"{module_name}\" (line {line})"
            ),
            path: None,
            line: Some(line),
            col: None,
        });
    }
    match disc.trim() {
        "rust" => Ok(JetImpl::Rust { symbol }),
        "bridge" => Ok(JetImpl::Bridge { symbol }),
        other => Err(ManifestError {
            code: ManifestErrorCode::UnknownImplDiscriminant,
            message: format!(
                "unknown __jet_impl discriminant {other:?} in module \"{module_name}\"; expected \"rust\" or \"bridge\" (line {line})"
            ),
            path: None,
            line: Some(line),
            col: None,
        }),
    }
}

fn take_identifier(s: &str) -> (String, &str) {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    (s[..end].to_string(), &s[end..])
}

fn strip_trailing_semicolon(s: &str) -> String {
    s.trim_end_matches(';').trim().to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn ancestor_chain(source_dir: &Path, workspace_root: &Path) -> Vec<PathBuf> {
    // Returns [source_dir, source_dir.parent(), …, workspace_root] inclusive.
    let mut out = Vec::new();
    let mut cursor = source_dir.to_path_buf();
    loop {
        out.push(cursor.clone());
        if cursor == workspace_root {
            break;
        }
        match cursor.parent() {
            Some(p) => cursor = p.to_path_buf(),
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_example() {
        let text = r#"
declare module "lodash/get" {
  export default function(obj: unknown, path: string): unknown;
  declare const __jet_impl: "rust:lodash_get";
}

declare module "@tanstack/react-query" {
  export function useQuery<T>(opts: UseQueryOpts): UseQueryResult<T>;
  declare const __jet_impl: "bridge:@tanstack/react-query";
}
"#;
        let entries = parse_manifest_text(text).unwrap();
        assert_eq!(entries.len(), 2);

        let lodash = &entries[0];
        assert_eq!(lodash.module_name, "lodash/get");
        assert_eq!(lodash.exports.len(), 1);
        assert_eq!(lodash.exports[0].kind, ExportKind::Default);
        assert_eq!(lodash.exports[0].name, "default");
        assert_eq!(
            lodash.jet_impl,
            JetImpl::Rust {
                symbol: "lodash_get".to_string()
            }
        );

        let rq = &entries[1];
        assert_eq!(rq.module_name, "@tanstack/react-query");
        assert_eq!(rq.exports.len(), 1);
        assert_eq!(rq.exports[0].kind, ExportKind::Named);
        assert_eq!(rq.exports[0].name, "useQuery");
        assert_eq!(
            rq.jet_impl,
            JetImpl::Bridge {
                symbol: "@tanstack/react-query".to_string()
            }
        );
    }

    #[test]
    fn export_const_no_signature() {
        let text = r#"
declare module "version" {
  export const VERSION: string;
  declare const __jet_impl: "rust:version_module";
}
"#;
        let entries = parse_manifest_text(text).unwrap();
        assert_eq!(entries[0].exports[0].kind, ExportKind::Named);
        assert_eq!(entries[0].exports[0].name, "VERSION");
        assert_eq!(entries[0].exports[0].signature, None);
    }

    #[test]
    fn unknown_impl_discriminant_rejected() {
        let text = r#"
declare module "x" {
  export default function(): void;
  declare const __jet_impl: "wasm:x";
}
"#;
        let err = parse_manifest_text(text).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::UnknownImplDiscriminant);
    }

    #[test]
    fn missing_jet_impl_rejected() {
        let text = r#"
declare module "x" {
  export default function(): void;
}
"#;
        let err = parse_manifest_text(text).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::ParseError);
        assert!(err.message.contains("missing `declare const __jet_impl"));
    }

    #[test]
    fn duplicate_module_in_one_file() {
        let text = r#"
declare module "x" {
  export default function(): void;
  declare const __jet_impl: "rust:x1";
}
declare module "x" {
  export default function(): void;
  declare const __jet_impl: "rust:x2";
}
"#;
        let err = parse_manifest_text(text).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::DuplicateModule);
    }

    #[test]
    fn export_default_arrow_rejected() {
        let text = r#"
declare module "x" {
  export default () => void;
  declare const __jet_impl: "rust:x";
}
"#;
        let err = parse_manifest_text(text).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::ParseError);
        assert!(err.message.contains("G-reject"));
    }

    #[test]
    fn nested_module_rejected() {
        let text = r#"
declare module "outer" {
  declare module "inner" {
    declare const __jet_impl: "rust:inner";
  }
  declare const __jet_impl: "rust:outer";
}
"#;
        let err = parse_manifest_text(text).unwrap_err();
        assert!(err.message.contains("G-reject-4") || err.message.contains("nested"));
    }

    #[test]
    fn import_statement_rejected() {
        let text = r#"
declare module "x" {
  import { Y } from "y";
  declare const __jet_impl: "rust:x";
}
"#;
        let err = parse_manifest_text(text).unwrap_err();
        assert!(err.message.contains("G-reject-2") || err.message.contains("import"));
    }

    #[test]
    fn comments_and_blanks_skipped() {
        let text = r#"
// top-level line comment
declare module "x" {
  // inside-block comment
  export default function(): void;
  declare const __jet_impl: "rust:x";
}
"#;
        let entries = parse_manifest_text(text).unwrap();
        assert_eq!(entries.len(), 1);
    }
}
// CODEGEN-END
