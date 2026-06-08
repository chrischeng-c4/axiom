//! Typeshed surface coverage tool.
//!
//! Compares the public Python-level surface that mamba's stdlib shim
//! registers for a given module (via `attrs.insert("<name>".to_string(), ...)`
//! inside `runtime/stdlib/<lib>_mod.rs`) against the corresponding typeshed
//! stub (`vendor/typeshed/stdlib/<lib>.pyi` or `stdlib/<lib>/__init__.pyi`
//! for stdlib; `vendor/typeshed/stubs/<lib>/<lib>.pyi` or
//! `stubs/<lib>/__init__.pyi` for third-party).
//!
//! Emits:
//!   implemented/total = N/M (P%)
//!     Missing: <comma list>
//!
//! This is a Phase 1.D conformance support tool — it does not attempt full
//! Python-grammar parsing. Resolution order (#2112):
//!   1. `__all__ = [...]` / `__all__ += [...]` / typed / tuple forms.
//!   2. Otherwise: top-level `def NAME(` / `async def NAME(` /
//!      `class NAME(` / `class NAME:` lines plus `from X import name as
//!      name` explicit re-exports.
//!   3. Companion `_<X>.pyi` stubs are unioned in for any top-level
//!      `from _<X> import *` line in the public stub.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Result of a surface comparison.
pub struct SurfaceReport {
    pub package: String,
    pub stub_path: PathBuf,
    pub mod_path: PathBuf,
    pub expected: BTreeSet<String>,
    pub implemented: BTreeSet<String>,
}

impl SurfaceReport {
    pub fn total(&self) -> usize { self.expected.len() }

    pub fn covered(&self) -> usize {
        self.expected.intersection(&self.implemented).count()
    }

    pub fn missing(&self) -> Vec<String> {
        self.expected
            .difference(&self.implemented)
            .cloned()
            .collect()
    }

    pub fn render(&self) -> String {
        let total = self.total();
        let n = self.covered();
        let pct = if total == 0 { 0.0 } else { (n as f64 / total as f64) * 100.0 };
        let mut out = format!("implemented/total = {}/{} ({:.0}%)\n", n, total, pct);
        let missing = self.missing();
        if missing.is_empty() {
            out.push_str("  Missing: (none)\n");
        } else {
            out.push_str(&format!("  Missing: {}\n", missing.join(", ")));
        }
        out
    }
}

/// Locate the typeshed stub for `package`.
///
/// Tries (in order): stdlib single-file, stdlib package `__init__.pyi`,
/// 3p `stubs/<pkg>/<pkg>.pyi`, 3p `stubs/<pkg>/__init__.pyi`.
pub fn find_stub(typeshed_root: &Path, package: &str) -> Option<PathBuf> {
    let candidates = [
        typeshed_root.join("stdlib").join(format!("{}.pyi", package)),
        typeshed_root.join("stdlib").join(package).join("__init__.pyi"),
        typeshed_root.join("stubs").join(package).join(format!("{}.pyi", package)),
        typeshed_root.join("stubs").join(package).join("__init__.pyi"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// Locate the mamba `<lib>_mod.rs` shim. Falls back to a `3p/<lib>_mod.rs`
/// path under the same stdlib directory.
pub fn find_mod_file(mamba_src: &Path, package: &str) -> Option<PathBuf> {
    let stdlib = mamba_src.join("runtime").join("stdlib");
    let stdlib_candidate = stdlib.join(format!("{}_mod.rs", package));
    if stdlib_candidate.is_file() {
        return Some(stdlib_candidate);
    }
    let third_party = stdlib.join("3p").join(format!("{}_mod.rs", package));
    if third_party.is_file() {
        return Some(third_party);
    }
    None
}

/// Parse a `.pyi` stub for the public surface (#2112).
///
/// Resolution order:
/// 1. If the stub declares `__all__ = [...]` (or `__all__: list[str] = [...]`)
///    use that list verbatim. Also picks up `__all__ += [...]` additions
///    (typeshed uses these for version-gated names like `binomialvariate` /
///    `call`). This is the explicit public contract — most accurate.
/// 2. Otherwise fall back to top-level `def NAME(` / `async def NAME(` /
///    `class NAME(` / `class NAME:` lines AND `from X import name as name`
///    re-export lines (sub-gap 2 — hashlib uses this shape with no `__all__`).
///
/// Private names (`_foo`) and dunders (`__foo__`) are filtered out in the
/// def/class fallback path; `__all__` is used as-authored.
///
/// Sub-gap 3 — companion `_<lib>.pyi` private stubs: when the public stub
/// does `from _<X> import *`, the wildcard contributes the public surface
/// of the companion stub (filtered to non-private names). This is unioned
/// into the resolved set regardless of whether `__all__` was present.
pub fn parse_stub(path: &Path) -> std::io::Result<BTreeSet<String>> {
    let text = fs::read_to_string(path)?;
    let mut names = if let Some(all_names) = parse_all_list(&text) {
        all_names
    } else {
        parse_defs_classes_and_reexports(&text)
    };
    // Sub-gap 3: union in names from `from _<X> import *` companion stubs
    // found in the same directory as `path`. Walks one level only — the
    // companion stub's own wildcard imports are not followed transitively.
    if let Some(dir) = path.parent() {
        for module in star_import_modules(&text) {
            if let Some(companion) = find_companion_stub(dir, &module) {
                if let Ok(companion_text) = fs::read_to_string(&companion) {
                    let companion_names = if let Some(all_names) =
                        parse_all_list(&companion_text)
                    {
                        all_names
                    } else {
                        parse_defs_classes_and_reexports(&companion_text)
                    };
                    for n in companion_names {
                        if !is_private(&n) {
                            names.insert(n);
                        }
                    }
                }
            }
        }
    }
    Ok(names)
}

/// Extract top-level `from <module> import *` modules from `text`.
/// Used by sub-gap 3 to discover companion stubs to consult.
fn star_import_modules(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.lines() {
        if raw.starts_with(' ') || raw.starts_with('\t') {
            continue;
        }
        let line = raw.trim_end();
        let Some(rest) = line.strip_prefix("from ") else { continue };
        let Some(import_idx) = rest.find(" import ") else { continue };
        let module = rest[..import_idx].trim();
        let tail = rest[import_idx + " import ".len()..].trim();
        if tail == "*" && !module.is_empty() {
            out.push(module.to_string());
        }
    }
    out
}

/// Resolve a module name to a `.pyi` path in `dir`. Supports the simple
/// `_<X>` and `<X>` single-segment forms used by typeshed companions
/// (e.g. `_operator` → `_operator.pyi`). Dotted modules are not resolved
/// here — typeshed companions live next to their public stub.
fn find_companion_stub(dir: &Path, module: &str) -> Option<PathBuf> {
    if module.contains('.') {
        return None;
    }
    let candidate = dir.join(format!("{}.pyi", module));
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

/// Extract names from `__all__ = [...]` / `__all__: list[str] = [...]` /
/// `__all__ += [...]`. Returns None when the stub doesn't declare `__all__`.
///
/// Handles:
/// - single-line: `__all__ = ["a", "b", "c"]`
/// - multi-line: `__all__ = [\n    "a",\n    "b",\n]`
/// - typed:      `__all__: list[str] = [...]`
/// - additions:  `__all__ += ["x"]` (under `if sys.version_info >= (...)`)
/// - tuple:      `__all__ = ("a", "b")`
fn parse_all_list(text: &str) -> Option<BTreeSet<String>> {
    let mut found_assignment = false;
    let mut names = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find the next line start.
        let line_start = i;
        // Find the end of the logical line (we'll re-scan token-by-token).
        let line_end = text[i..].find('\n').map(|d| i + d).unwrap_or(bytes.len());
        let raw_line = &text[line_start..line_end];

        // Trim leading whitespace once to test for `__all__` keyword.
        let trimmed = raw_line.trim_start();
        let is_top_level = raw_line.len() == trimmed.len()
            || raw_line.starts_with('#');

        let starts_all = trimmed.starts_with("__all__");
        if starts_all && is_top_level {
            // Consume from `__all__` up to the opening bracket `[` or `(`.
            // Accept forms: `__all__ = [`, `__all__: list[str] = [`,
            //               `__all__ += [`, `__all__ = (`.
            // Find the matching opener after `=` (could be on same or next line
            // but typeshed always keeps the opener on the same line as `=`).
            if let Some(eq_idx) = trimmed.find('=') {
                let after_eq = trimmed[eq_idx + 1..].trim_start();
                if let Some(opener) = after_eq.chars().next() {
                    if opener == '[' || opener == '(' {
                        let closer = if opener == '[' { ']' } else { ')' };
                        found_assignment = true;
                        // Walk forward from this point until we hit the
                        // matching `closer` (may span many lines). Collect
                        // every `"..."` or `'...'` literal we see.
                        // Search for `opener` AFTER the `=`, not from the
                        // start of `trimmed` — otherwise a typed annotation
                        // like `__all__: list[str] = [...]` matches the `[`
                        // inside `list[str]` and we never reach the real
                        // opener. (Fixed for #2112 multi-line+typed test.)
                        let rel_after_eq = trimmed[eq_idx + 1..]
                            .find(opener)
                            .expect("opener present after `=`");
                        let open_pos_in_trimmed = eq_idx + 1 + rel_after_eq;
                        let open_abs = line_start
                            + (raw_line.len() - trimmed.len())
                            + open_pos_in_trimmed
                            + 1; // skip past opener
                        let close_abs = find_matching(
                            &text[open_abs..],
                            opener,
                            closer,
                        )
                        .map(|d| open_abs + d)
                        .unwrap_or(bytes.len());
                        collect_string_literals(
                            &text[open_abs..close_abs],
                            &mut names,
                        );
                        // Advance past the closer.
                        i = close_abs + 1;
                        continue;
                    }
                }
            }
        }
        i = line_end + 1;
    }
    if found_assignment {
        Some(names)
    } else {
        None
    }
}

/// Find the offset of `closer` in `s`, balancing nested `opener`/`closer`
/// while ignoring brackets that appear inside `"..."` or `'...'` literals.
fn find_matching(s: &str, opener: char, closer: char) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth: i32 = 1; // we're already past the first opener
    let mut in_str: Option<u8> = None;
    let mut esc = false;
    for (i, &b) in bytes.iter().enumerate() {
        if let Some(q) = in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == q {
                in_str = None;
            }
            continue;
        }
        match b {
            b'"' | b'\'' => in_str = Some(b),
            b'#' => {
                // Skip rest of the line — Python comment.
                if let Some(d) = s[i..].find('\n') {
                    return s[i + d + 1..]
                        .find(closer)
                        .map(|d2| i + d + 1 + d2);
                }
                return None;
            }
            _ if b as char == opener => depth += 1,
            _ if b as char == closer => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Pull every `"..."` / `'...'` string literal out of `s` and add identifiers
/// to `names`. Non-identifier strings (e.g. partial / weird) are skipped.
fn collect_string_literals(s: &str, names: &mut BTreeSet<String>) {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // Skip Python comments — to end-of-line.
        if b == b'#' {
            i += s[i..].find('\n').map(|d| d + 1).unwrap_or(bytes.len() - i);
            continue;
        }
        if b == b'"' || b == b'\'' {
            let quote = b;
            let start = i + 1;
            let mut j = start;
            let mut esc = false;
            while j < bytes.len() {
                if esc {
                    esc = false;
                } else if bytes[j] == b'\\' {
                    esc = true;
                } else if bytes[j] == quote {
                    break;
                }
                j += 1;
            }
            if j < bytes.len() {
                let candidate = &s[start..j];
                if is_identifier(candidate) {
                    names.insert(candidate.to_string());
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
}

/// Fallback when `__all__` is absent: top-level defs/classes/async defs
/// plus `from X import name as name` re-exports (sub-gap 2 of #2112 —
/// hashlib uses this shape with no `__all__`).
fn parse_defs_classes_and_reexports(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        let line_end = text[i..].find('\n').map(|d| i + d).unwrap_or(bytes.len());
        let raw = &text[i..line_end];
        // Only top-level — no leading whitespace.
        let is_indented = raw.starts_with(' ') || raw.starts_with('\t');
        if !is_indented {
            let line = raw.trim_end();
            if let Some(rest) = line.strip_prefix("def ") {
                if let Some(name) = extract_ident(rest, '(') {
                    if !is_private(&name) {
                        names.insert(name);
                    }
                }
            } else if let Some(rest) = line.strip_prefix("async def ") {
                if let Some(name) = extract_ident(rest, '(') {
                    if !is_private(&name) {
                        names.insert(name);
                    }
                }
            } else if let Some(rest) = line.strip_prefix("class ") {
                let stop = rest.find(|c: char| c == '(' || c == ':');
                if let Some(idx) = stop {
                    let name = rest[..idx].trim().to_string();
                    if !name.is_empty() && !is_private(&name) {
                        names.insert(name);
                    }
                }
            } else if line.starts_with("from ") && line.contains(" import ") {
                // `from X import a as a, b as b, ...` — possibly multi-line
                // with `(` / `)`. Find the import list span first.
                let (span, advanced) = if line.contains('(') {
                    // Multi-line: gather until matching `)`.
                    let open_abs = i + raw.find('(').unwrap() + 1;
                    let close_rel = find_matching(&text[open_abs..], '(', ')');
                    match close_rel {
                        Some(d) => {
                            let close_abs = open_abs + d;
                            let s = text[open_abs..close_abs].to_string();
                            i = close_abs + 1;
                            (s, true)
                        }
                        None => {
                            i = line_end + 1;
                            continue;
                        }
                    }
                } else if let Some(after) = line.find(" import ") {
                    (line[after + " import ".len()..].to_string(), false)
                } else {
                    (String::new(), false)
                };
                collect_reexports(&span, &mut names);
                if advanced {
                    // Multi-line case already advanced `i`; loop without
                    // bumping past the line_end (which would skip content).
                    continue;
                }
                // Single-line case — fall through to the normal advance.
            }
        }
        i = line_end + 1;
    }
    names
}

/// From `a as a, b as b, c, d as alias` extract names where alias == name
/// (typeshed's "explicit re-export" convention). Bare imports (no alias)
/// are NOT collected — typeshed marks intentional re-exports with the
/// `name as name` form.
fn collect_reexports(span: &str, names: &mut BTreeSet<String>) {
    for piece in span.split(',') {
        let piece = piece.trim().trim_end_matches(',');
        if let Some(as_idx) = piece.find(" as ") {
            let lhs = piece[..as_idx].trim();
            let rhs = piece[as_idx + 4..].trim();
            if lhs == rhs && is_identifier(lhs) && !is_private(lhs) {
                names.insert(lhs.to_string());
            }
        }
    }
}

/// Parse a mamba `*_mod.rs` for the Python-level names it registers.
///
/// Matches:
///   `attrs.insert("<name>".to_string(),`
/// and for `os.path`-style submodule tables (which `os_mod.rs` uses):
///   `path_attrs.insert("<name>".to_string(),`
/// Also matches the (`"<name>"`, dispatch_xxx as ...) tuple-list form via
/// the same regex (the leading `"..."` literal is what we anchor on).
pub fn parse_mod(path: &Path) -> std::io::Result<BTreeSet<String>> {
    let text = fs::read_to_string(path)?;
    let mut names = BTreeSet::new();
    // Pattern 1: <ident>.insert("<name>".to_string(), ...)
    // Pattern 2: ("<name>", dispatch_xxx as *const () ...)  — used by os_mod's
    //            dispatcher tuple list.
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = pick_attrs_insert(trimmed) {
            names.insert(name);
        } else if let Some(name) = pick_tuple_dispatcher(trimmed) {
            names.insert(name);
        }
    }
    Ok(names)
}

fn pick_attrs_insert(line: &str) -> Option<String> {
    // `<ident>.insert("<name>".to_string(),` — anchor on `.insert("`.
    // Also accepts `.into()` as the conversion verb — some shims (json_mod)
    // use `.into()` for static names like `JSONDecodeError`.
    let idx = line.find(".insert(\"")?;
    let after = &line[idx + ".insert(\"".len()..];
    let end = after.find('"')?;
    let name = &after[..end];
    let rest = &after[end + 1..];
    let rest_trim = rest.trim_start();
    if !rest_trim.starts_with(".to_string()") && !rest_trim.starts_with(".into()") {
        return None;
    }
    if name.is_empty() || !is_identifier(name) {
        return None;
    }
    Some(name.to_string())
}

fn pick_tuple_dispatcher(line: &str) -> Option<String> {
    // `("<name>", dispatch_xxx as *const () as usize),`
    let trimmed = line.trim_start_matches(|c: char| c.is_whitespace());
    let after = trimmed.strip_prefix("(\"")?;
    let end = after.find('"')?;
    let name = &after[..end];
    let rest = &after[end + 1..];
    // Comma then a `dispatch_` identifier-style — fine to be permissive here.
    let rest = rest.trim_start_matches(',').trim_start();
    if !rest.starts_with("dispatch_") {
        return None;
    }
    if name.is_empty() || !is_identifier(name) {
        return None;
    }
    Some(name.to_string())
}

fn extract_ident(after_keyword: &str, terminator: char) -> Option<String> {
    let end = after_keyword.find(terminator)?;
    let candidate = after_keyword[..end].trim();
    if candidate.is_empty() { None } else { Some(candidate.to_string()) }
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_private(name: &str) -> bool {
    name.starts_with('_')
}

/// Build a surface report for `package`. Tries stdlib first, then 3p.
pub fn build_report(
    package: &str,
    typeshed_root: &Path,
    mamba_src: &Path,
) -> Result<SurfaceReport, String> {
    let stub_path = find_stub(typeshed_root, package).ok_or_else(|| {
        format!(
            "typeshed stub not found for `{}` under {}",
            package,
            typeshed_root.display()
        )
    })?;
    let mod_path = find_mod_file(mamba_src, package);

    let expected = parse_stub(&stub_path)
        .map_err(|e| format!("failed to read stub {}: {}", stub_path.display(), e))?;

    let implemented = if let Some(ref p) = mod_path {
        parse_mod(p).map_err(|e| {
            format!("failed to read mod {}: {}", p.display(), e)
        })?
    } else {
        BTreeSet::new()
    };

    Ok(SurfaceReport {
        package: package.to_string(),
        stub_path,
        mod_path: mod_path.unwrap_or_else(|| PathBuf::from("(not implemented)")),
        expected,
        implemented,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmpdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn parse_stub_picks_top_level_defs_and_classes() {
        let d = tmpdir();
        let stub = d.path().join("foo.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        writeln!(f, "def getcwd() -> str: ...").unwrap();
        writeln!(f, "def listdir(path: str = '.') -> list[str]: ...").unwrap();
        writeln!(f, "class PathLike:").unwrap();
        writeln!(f, "    def method(self): ...").unwrap();
        writeln!(f, "    def _private(self): ...").unwrap();
        writeln!(f, "def _hidden() -> None: ...").unwrap();
        writeln!(f, "async def coro() -> int: ...").unwrap();
        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("getcwd"));
        assert!(names.contains("listdir"));
        assert!(names.contains("PathLike"));
        assert!(names.contains("coro"));
        assert!(!names.contains("_hidden"));
        assert!(!names.contains("method"));
        assert!(!names.contains("_private"));
    }

    #[test]
    fn parse_mod_picks_attrs_insert_and_tuple_dispatcher() {
        let d = tmpdir();
        let modf = d.path().join("foo_mod.rs");
        let mut f = fs::File::create(&modf).unwrap();
        writeln!(f, "attrs.insert(\"getcwd\".to_string(), MbValue::from_func(0));").unwrap();
        writeln!(f, "attrs.insert(\"listdir\".to_string(),").unwrap();
        writeln!(f, "    MbValue::from_func(addr));").unwrap();
        writeln!(f, "    (\"mkdir\", dispatch_mkdir as *const () as usize),").unwrap();
        writeln!(f, "    // commented (\"junk\", dispatch_junk as ..)").unwrap();
        let names = parse_mod(&modf).unwrap();
        assert!(names.contains("getcwd"));
        assert!(names.contains("listdir"));
        assert!(names.contains("mkdir"));
    }

    /// Acceptance test for Phase 1.D — runs the surface report against
    /// the real vendored typeshed + mamba's `os_mod.rs` / (absent) `ssl_mod.rs`.
    /// Skipped automatically if typeshed isn't checked out.
    #[test]
    fn acceptance_os_and_ssl_report() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let typeshed = manifest_dir.join("vendor").join("typeshed");
        let mamba_src = manifest_dir.join("src");
        if !typeshed.is_dir() {
            eprintln!(
                "skipping: {} not present (run `git clone --depth=1 https://github.com/python/typeshed.git {}`)",
                typeshed.display(),
                typeshed.display()
            );
            return;
        }

        let os_report = build_report("os", &typeshed, &mamba_src).expect("os report");
        let ssl_report = build_report("ssl", &typeshed, &mamba_src).expect("ssl report");

        // Headline assertions — these are the Phase 1.D acceptance numbers.
        // os: at least a handful registered; ssl: not implemented yet.
        assert!(
            os_report.covered() >= 5,
            "os covered too few names: {}/{}",
            os_report.covered(),
            os_report.total()
        );
        assert!(
            os_report.total() >= 50,
            "os typeshed surface looks empty: total={}",
            os_report.total()
        );
        // ssl_mod.rs shipped as a surface-only shim under #1414. It is not
        // yet a functional TLS implementation, but it does register a
        // sizeable Gate 2 attribute surface (≥10 names) so the requests /
        // urllib3 / httpx import-time probe chain resolves. Treat any
        // non-zero coverage as acceptable — the assertion here is about
        // the surface-report plumbing, not the TLS roadmap.
        assert!(
            ssl_report.covered() >= 5,
            "ssl_mod.rs shipped — covered should be non-trivial, got {}",
            ssl_report.covered()
        );
        assert!(
            ssl_report.total() >= 10,
            "ssl typeshed surface looks empty: total={}",
            ssl_report.total()
        );

        eprintln!("Acceptance ({}): {}", "os", os_report.render().trim_end());
        eprintln!("Acceptance ({}): {}", "ssl", ssl_report.render().trim_end());
    }

    #[test]
    fn parse_stub_picks_all_list_single_line() {
        let d = tmpdir();
        let stub = d.path().join("foo.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        writeln!(f, "__all__ = [\"alpha\", \"beta\", \"gamma\"]").unwrap();
        writeln!(f, "def hidden_helper() -> None: ...").unwrap();
        writeln!(f, "class HiddenClass: ...").unwrap();
        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("alpha"));
        assert!(names.contains("beta"));
        assert!(names.contains("gamma"));
        // When __all__ is declared, it IS the contract — bare defs/classes
        // are NOT unioned in.
        assert!(!names.contains("hidden_helper"));
        assert!(!names.contains("HiddenClass"));
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn parse_stub_picks_all_list_multi_line_and_typed() {
        let d = tmpdir();
        let stub = d.path().join("foo.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        writeln!(f, "__all__: list[str] = [").unwrap();
        writeln!(f, "    \"abs\",").unwrap();
        writeln!(f, "    \"add\",").unwrap();
        writeln!(f, "    \"call\",").unwrap();
        writeln!(f, "]").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "if sys.version_info >= (3, 11):").unwrap();
        writeln!(f, "    __all__ += [\"call\"]").unwrap();
        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("abs"));
        assert!(names.contains("add"));
        assert!(names.contains("call"));
    }

    #[test]
    fn parse_stub_picks_all_list_tuple_form() {
        let d = tmpdir();
        let stub = d.path().join("foo.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        writeln!(f, "__all__ = (\"x\", \"y\", \"z\")").unwrap();
        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("x"));
        assert!(names.contains("y"));
        assert!(names.contains("z"));
    }

    #[test]
    fn parse_stub_follows_explicit_reexports() {
        // No `__all__` declared — fallback path should pick up `name as name`
        // re-exports (typeshed's explicit re-export convention).
        let d = tmpdir();
        let stub = d.path().join("foo.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        writeln!(f, "from _impl import abs as abs, add as add, sub as sub").unwrap();
        writeln!(f, "from _impl import private_name  # bare — NOT re-exported").unwrap();
        writeln!(f, "from _impl import (").unwrap();
        writeln!(f, "    mul as mul,").unwrap();
        writeln!(f, "    div as div,").unwrap();
        writeln!(f, ")").unwrap();
        writeln!(f, "def local_fn() -> None: ...").unwrap();
        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("abs"));
        assert!(names.contains("add"));
        assert!(names.contains("sub"));
        assert!(names.contains("mul"));
        assert!(names.contains("div"));
        assert!(names.contains("local_fn"));
        assert!(!names.contains("private_name"));
    }

    #[test]
    fn parse_stub_unions_companion_private_stub_via_star_import() {
        // Sub-gap 3 of #2112 — `from _foo import *` should pull the public
        // surface of a sibling `_foo.pyi` into the importing module's
        // surface. Mirrors typeshed's `operator.pyi` + `_operator.pyi`
        // shape (though typeshed `operator.pyi` actually uses explicit
        // re-exports — the wildcard form is the case sub-gap 3 targets).
        let d = tmpdir();
        // Companion private stub.
        let companion = d.path().join("_foo.pyi");
        let mut cf = fs::File::create(&companion).unwrap();
        writeln!(cf, "def helper_a(x: int) -> int: ...").unwrap();
        writeln!(cf, "def helper_b(x: int) -> int: ...").unwrap();
        writeln!(cf, "class Helper: ...").unwrap();
        writeln!(cf, "def _hidden() -> None: ...").unwrap();
        drop(cf);

        // Public stub that wildcard-imports the companion.
        let stub = d.path().join("foo.pyi").to_path_buf();
        let mut sf = fs::File::create(&stub).unwrap();
        writeln!(sf, "from _foo import *").unwrap();
        writeln!(sf, "def local_fn() -> None: ...").unwrap();
        drop(sf);

        let names = parse_stub(&stub).unwrap();
        // Public stub's own def.
        assert!(names.contains("local_fn"));
        // Companion contributions via `*` import.
        assert!(names.contains("helper_a"));
        assert!(names.contains("helper_b"));
        assert!(names.contains("Helper"));
        // Private name in the companion stays filtered.
        assert!(!names.contains("_hidden"));
    }

    #[test]
    fn parse_stub_companion_respects_companion_all_list() {
        // When the companion declares `__all__`, that's the authoritative
        // public surface — we should not pick up names absent from it
        // (e.g. helper internals defined alongside `__all__`).
        let d = tmpdir();
        let companion = d.path().join("_bar.pyi");
        let mut cf = fs::File::create(&companion).unwrap();
        writeln!(cf, "__all__ = [\"keep_me\"]").unwrap();
        writeln!(cf, "def keep_me() -> None: ...").unwrap();
        writeln!(cf, "def drop_me() -> None: ...").unwrap();
        drop(cf);

        let stub = d.path().join("bar.pyi");
        let mut sf = fs::File::create(&stub).unwrap();
        writeln!(sf, "from _bar import *").unwrap();
        drop(sf);

        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("keep_me"));
        assert!(!names.contains("drop_me"));
    }

    #[test]
    fn parse_stub_companion_missing_is_silent() {
        // `from _missing import *` with no companion file present must
        // not error — we just don't union anything in.
        let d = tmpdir();
        let stub = d.path().join("baz.pyi");
        let mut sf = fs::File::create(&stub).unwrap();
        writeln!(sf, "from _missing import *").unwrap();
        writeln!(sf, "def local_fn() -> None: ...").unwrap();
        drop(sf);

        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("local_fn"));
        assert_eq!(names.len(), 1);
    }

    #[test]
    fn parse_stub_companion_unions_with_explicit_all() {
        // Even when the public stub declares __all__, a `from _X import *`
        // companion still contributes — `__all__` controls names exposed
        // from THIS file; star imports re-publish the companion's surface.
        let d = tmpdir();
        let companion = d.path().join("_qux.pyi");
        let mut cf = fs::File::create(&companion).unwrap();
        writeln!(cf, "def from_companion() -> None: ...").unwrap();
        drop(cf);

        let stub = d.path().join("qux.pyi");
        let mut sf = fs::File::create(&stub).unwrap();
        writeln!(sf, "__all__ = [\"declared\"]").unwrap();
        writeln!(sf, "from _qux import *").unwrap();
        writeln!(sf, "def declared() -> None: ...").unwrap();
        drop(sf);

        let names = parse_stub(&stub).unwrap();
        assert!(names.contains("declared"));
        assert!(names.contains("from_companion"));
    }

    #[test]
    fn parse_mod_accepts_into_conversion() {
        // json_mod registers JSONDecodeError via `.into()` instead of
        // `.to_string()`. We must recognize both.
        let d = tmpdir();
        let modf = d.path().join("foo_mod.rs");
        let mut f = fs::File::create(&modf).unwrap();
        writeln!(f, "attrs.insert(\"loads\".to_string(), MbValue::from_func(0));").unwrap();
        writeln!(f, "attrs.insert(\"JSONDecodeError\".into(), exc_class);").unwrap();
        let names = parse_mod(&modf).unwrap();
        assert!(names.contains("loads"));
        assert!(names.contains("JSONDecodeError"));
    }

    /// Regression for #2112 — walker must count ~26 names on a stub shaped
    /// like typeshed's `random.pyi`: `__all__` lists most of the public
    /// surface, while the module-level bindings are `name = _inst.method`
    /// re-assignments (NOT `def`s) that the fallback walker can't see.
    ///
    /// Before #2112: walker reported 0-2 (the two top-level classes only).
    /// After #2112: walker reports `__all__` verbatim — 26 here, matching
    /// CPython 3.12's `random` surface in `cpython312_surface.json`.
    #[test]
    fn parse_stub_random_shape_regression_2112() {
        let d = tmpdir();
        let stub = d.path().join("random.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        // Mirror typeshed's random.pyi __all__ — 26 names.
        writeln!(f, "__all__ = [").unwrap();
        for name in [
            "Random", "seed", "random", "uniform", "randint", "choice",
            "sample", "randrange", "shuffle", "normalvariate", "lognormvariate",
            "expovariate", "vonmisesvariate", "gammavariate", "triangular",
            "gauss", "betavariate", "paretovariate", "weibullvariate",
            "getstate", "setstate", "getrandbits", "choices", "SystemRandom",
            "randbytes", "binomialvariate",
        ] {
            writeln!(f, "    \"{}\",", name).unwrap();
        }
        writeln!(f, "]").unwrap();
        // Module-level `name = _inst.method` re-assignments — invisible to
        // the def/class fallback. These should NOT be required to be picked
        // up; `__all__` is the contract.
        writeln!(f, "_inst: Random").unwrap();
        writeln!(f, "seed = _inst.seed").unwrap();
        writeln!(f, "random = _inst.random").unwrap();
        writeln!(f, "uniform = _inst.uniform").unwrap();
        writeln!(f, "class Random: ...").unwrap();
        writeln!(f, "class SystemRandom(Random): ...").unwrap();

        let names = parse_stub(&stub).unwrap();
        // Headline assertion: matches CPython 3.12 random surface (26).
        assert_eq!(
            names.len(),
            26,
            "expected 26 names from random-shaped __all__; got {}: {:?}",
            names.len(),
            names
        );
        // Spot-check: must include re-export-style names that the def/class
        // fallback cannot see.
        for must in ["seed", "random", "uniform", "getrandbits", "binomialvariate"] {
            assert!(names.contains(must), "missing {} from random surface", must);
        }
    }

    /// Regression for #2112 sub-gap 2 — operator.pyi has no `__all__` but
    /// re-exports ~55 names via `from _operator import name as name`. The
    /// fallback path must surface those via `collect_reexports`.
    ///
    /// Before #2112: walker saw 3 classes (`attrgetter` / `itemgetter` /
    /// `methodcaller`). After: walker sees 3 + ~55 = ~58 names.
    #[test]
    fn parse_stub_operator_shape_regression_2112() {
        let d = tmpdir();
        let stub = d.path().join("operator.pyi");
        let mut f = fs::File::create(&stub).unwrap();
        writeln!(f, "from _operator import (").unwrap();
        let reexports = [
            "abs", "add", "and_", "concat", "contains", "countOf", "delitem",
            "eq", "floordiv", "ge", "getitem", "gt", "iadd", "iand", "iconcat",
            "ifloordiv", "ilshift", "imatmul", "imod", "imul", "indexOf",
            "inv", "invert", "ior", "ipow", "irshift", "is_", "is_not",
            "isub", "itruediv", "ixor", "le", "lshift", "lt", "matmul", "mod",
            "mul", "ne", "neg", "not_", "or_", "pos", "pow", "rshift",
            "setitem", "sub", "truediv", "truth", "xor",
        ];
        for name in reexports {
            writeln!(f, "    {} as {},", name, name).unwrap();
        }
        writeln!(f, ")").unwrap();
        writeln!(f, "class attrgetter: ...").unwrap();
        writeln!(f, "class itemgetter: ...").unwrap();
        writeln!(f, "class methodcaller: ...").unwrap();

        let names = parse_stub(&stub).unwrap();
        let expected = reexports.len() + 3;
        assert_eq!(
            names.len(),
            expected,
            "expected {} names; got {}: {:?}",
            expected,
            names.len(),
            names
        );
        for cls in ["attrgetter", "itemgetter", "methodcaller"] {
            assert!(names.contains(cls), "missing class {}", cls);
        }
        for fnname in ["abs", "add", "truediv", "xor"] {
            assert!(names.contains(fnname), "missing re-export {}", fnname);
        }
    }

    #[test]
    fn report_diffs_expected_and_implemented() {
        let report = SurfaceReport {
            package: "demo".to_string(),
            stub_path: PathBuf::from("/dev/null"),
            mod_path: PathBuf::from("/dev/null"),
            expected: ["a", "b", "c"].iter().map(|s| s.to_string()).collect(),
            implemented: ["a", "b", "z"].iter().map(|s| s.to_string()).collect(),
        };
        assert_eq!(report.total(), 3);
        assert_eq!(report.covered(), 2);
        assert_eq!(report.missing(), vec!["c".to_string()]);
        let rendered = report.render();
        assert!(rendered.contains("2/3"));
        assert!(rendered.contains("(67%)"));
        assert!(rendered.contains("Missing: c"));
    }
}
