// `uv pip install -r requirements.txt` / `uv pip compile`-style parser (Tick 47).
//
// Pure-text inverse of `requirements_export` (Tick 41). Turns a
// requirements.txt body into a stream of typed `RequirementLine`s without
// touching the filesystem. Recursive `-r` / `-c` includes are returned as
// `RequirementLine::Include` / `Constraint` items; the caller decides whether
// to follow them.
//
// Recognised forms (pip / PEP 508 surface):
//   * `# comment`, ` # trailing comment` — stripped.
//   * Line continuation: a trailing `\` joins with the next physical line.
//   * `-r path/to/other.txt`, `--requirement path/to/other.txt` → Include.
//   * `-c path/to/constraints.txt`, `--constraint ...` → Constraint.
//   * `-e <url-or-path>`, `--editable <url-or-path>` → Editable.
//   * `name`, `name==1.2.3`, `name>=1,<2`, `name[extra1,extra2]==1.2.3`.
//   * `name @ <url>` direct-URL refs (git+/file://+/https://, etc).
//   * `; python_version >= "3.10" and sys_platform == "linux"` markers.
//   * `--hash=sha256:<hex>` continuations (one per line after the spec).
//
// Tick 71 (added) — pip index/binary flags. The following directives now
// promote out of `RequirementLine::Unknown` into a typed
// `RequirementLine::IndexFlag(IndexFlag)` so the resolver can consume them:
//   `--index-url`, `--extra-index-url`, `--find-links`, `--no-index`,
//   `--no-binary`, `--only-binary`, `--prefer-binary`, `--pre`,
//   `--trusted-host`.
//
// Out of scope (deferred):
//   * Environment-variable expansion (`${VAR}`) — caller's responsibility.
//   * Full PEP 508 marker evaluation — markers are kept as opaque strings.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One logical line in a requirements.txt file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementLine {
    /// `-r path/to/other.txt` — recursive include.
    Include(String),
    /// `-c path/to/constraints.txt` — constraint file.
    Constraint(String),
    /// `-e <url-or-path>` — editable install.
    Editable(EditableSpec),
    /// A normal package requirement (PEP 508).
    Package(PackageRequirement),
    /// One of pip's index/binary directives — `--index-url`, `--no-binary`,
    /// etc. See [`IndexFlag`] for the variants. Promoted out of
    /// [`RequirementLine::Unknown`] in Tick 71 so the resolver can act on
    /// them rather than passing the raw line through.
    IndexFlag(IndexFlag),
    /// A well-formed but unrecognised `-`/`--`-prefixed directive.
    /// The raw text is preserved verbatim so callers can pass it through.
    Unknown(String),
}

/// pip-compatible index/binary directives parsed from a requirements.txt
/// stream. uv's resolver consumes the same set; we mirror only the surface
/// pip documents at <https://pip.pypa.io/en/stable/reference/requirements-file-format/>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexFlag {
    /// `--index-url URL` (also `-i URL`). Replaces the default PyPI index.
    IndexUrl(String),
    /// `--extra-index-url URL`. Adds a fallback index.
    ExtraIndexUrl(String),
    /// `--find-links URL_OR_PATH`. Local/remote location to scan for archives.
    FindLinks(String),
    /// `--no-index`. Disable the canonical index lookup entirely.
    NoIndex,
    /// `--no-binary NAME[,NAME...]` or `--no-binary :all:` / `:none:`.
    /// The comma-separated payload is preserved as written.
    NoBinary(String),
    /// `--only-binary NAME[,NAME...]` (same payload shape as `NoBinary`).
    OnlyBinary(String),
    /// `--prefer-binary`. Prefer wheels even when an older one tags better.
    PreferBinary,
    /// `--pre`. Allow pre-release versions during resolution.
    Pre,
    /// `--trusted-host HOST`. Disable TLS verification for HOST.
    TrustedHost(String),
}

/// Editable install reference. Mirrors `pip install -e <url-or-path>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableSpec {
    /// The url-or-path argument as written (post-strip, no quoting).
    pub target: String,
    /// `[extras]` if the user wrote `-e .[dev,test]`.
    pub extras: Vec<String>,
}

/// One parsed package requirement.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageRequirement {
    /// Project name, as written (case preserved). Use `pep503_normalize`
    /// when comparing.
    pub name: String,
    /// `[extra1,extra2]` extras, in source order.
    pub extras: Vec<String>,
    /// PEP 440 version specifiers, e.g. `[">=1.0", "<2"]`. Empty for an
    /// unconstrained or direct-URL requirement.
    pub specifiers: Vec<String>,
    /// PEP 508 marker expression (post-`;`), trimmed. `None` when absent.
    pub marker: Option<String>,
    /// Direct-URL reference (post-`@`), trimmed. `None` for a registry pin.
    pub direct_url: Option<String>,
    /// `--hash=sha256:<hex>` annotations, in source order.
    pub hashes: Vec<String>,
}

/// Top-level entry point: parse a full requirements.txt body.
///
/// Returns a structured stream preserving source order. Blank lines and
/// pure-comment lines are dropped. Hash continuations attach to the most
/// recent package or editable line, mirroring pip's `--require-hashes` rule.
pub fn parse_requirements_txt(src: &str) -> Result<Vec<RequirementLine>, IndexError> {
    let mut out: Vec<RequirementLine> = Vec::new();

    for logical in fold_logical_lines(src) {
        let line = strip_comment(&logical).trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Hash continuations are attached to the preceding requirement so a
        // multi-line `name==1.2.3 \\ --hash=sha256:...` block lands on one
        // PackageRequirement / EditableSpec rather than splitting.
        if let Some(hash) = parse_hash_only(&line) {
            attach_hash(&mut out, hash)?;
            continue;
        }

        let parsed = parse_one_line(&line)?;
        out.push(parsed);
    }

    Ok(out)
}

/// Parse a single, already-comment-stripped, already-joined logical line.
pub fn parse_one_line(line: &str) -> Result<RequirementLine, IndexError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return parse_err("empty requirement line");
    }

    // Flag-style directives. `-r foo` / `--requirement foo`, etc.
    if let Some(rest) = strip_flag(trimmed, &["-r", "--requirement"]) {
        let path = rest.trim().to_string();
        if path.is_empty() {
            return parse_err("missing path after -r/--requirement");
        }
        return Ok(RequirementLine::Include(path));
    }
    if let Some(rest) = strip_flag(trimmed, &["-c", "--constraint"]) {
        let path = rest.trim().to_string();
        if path.is_empty() {
            return parse_err("missing path after -c/--constraint");
        }
        return Ok(RequirementLine::Constraint(path));
    }
    if let Some(rest) = strip_flag(trimmed, &["-e", "--editable"]) {
        let payload = rest.trim();
        if payload.is_empty() {
            return parse_err("missing target after -e/--editable");
        }
        return Ok(RequirementLine::Editable(parse_editable(payload)));
    }

    // pip-compatible index/binary directives. Each entry maps flag aliases
    // to a constructor closure that may return `None` (boolean flag with no
    // payload) or `Some(IndexFlag)` (payload-carrying flag).
    if let Some(flag) = parse_index_flag(trimmed)? {
        return Ok(RequirementLine::IndexFlag(flag));
    }

    // Unknown long-flag directives are kept verbatim.
    if trimmed.starts_with('-') {
        return Ok(RequirementLine::Unknown(trimmed.to_string()));
    }

    // A bare package line: `name[extras]<specs> [@ url] [; marker]
    // [--hash=...]?`. We pull off the hash tail first because hashes may
    // appear inline rather than on a continuation line.
    let (head, tail_hashes) = split_inline_hashes(trimmed);
    let pkg = parse_package_line(&head)?;
    let mut pkg = pkg;
    pkg.hashes.extend(tail_hashes);
    Ok(RequirementLine::Package(pkg))
}

// ---------------------------------------------------------------------------
// Logical-line folding & comment stripping
// ---------------------------------------------------------------------------

/// Join physical lines that end with `\` into single logical lines.
fn fold_logical_lines(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut continuing = false;
    for raw in src.lines() {
        let line = raw.trim_end_matches(['\r']);
        if let Some(stripped) = line.strip_suffix('\\') {
            if continuing {
                buf.push(' ');
            }
            buf.push_str(stripped.trim_end());
            continuing = true;
            continue;
        }
        if continuing {
            buf.push(' ');
            buf.push_str(line);
            out.push(std::mem::take(&mut buf));
            continuing = false;
        } else {
            out.push(line.to_string());
        }
    }
    if continuing {
        out.push(buf);
    }
    out
}

/// Remove a trailing `# comment`. A `#` inside `"..."` quoted strings
/// (markers) is preserved.
fn strip_comment(line: &str) -> String {
    let mut in_dq = false;
    let mut in_sq = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' if !in_sq => in_dq = !in_dq,
            '\'' if !in_dq => in_sq = !in_sq,
            '#' if !in_dq && !in_sq => {
                return line[..i].to_string();
            }
            _ => {}
        }
    }
    line.to_string()
}

// ---------------------------------------------------------------------------
// Flag detection
// ---------------------------------------------------------------------------

/// If `line` starts with one of `flags` followed by whitespace or `=`, return
/// the rest of the line after that flag. Otherwise `None`.
fn strip_flag<'a>(line: &'a str, flags: &[&str]) -> Option<&'a str> {
    for f in flags {
        if let Some(rest) = line.strip_prefix(f) {
            // `-r foo`, `--requirement foo`, `--requirement=foo`.
            if rest.is_empty() {
                return Some("");
            }
            let next = rest.chars().next().unwrap();
            if next == ' ' || next == '\t' || next == '=' {
                return Some(&rest[1..]);
            }
            // `-rfoo` is intentionally not supported — pip treats it as
            // shorthand but it conflicts with `-r` being a flag here.
        }
    }
    None
}

/// Recognize one of the pip-compatible index/binary directives. Returns
/// `Ok(Some(...))` if the line matches a known flag, `Ok(None)` if it
/// looks like an unrelated directive (let the caller fall back to
/// `Unknown`), and `Err(...)` if the flag matched but had a malformed
/// payload (e.g. `--index-url` with no URL).
fn parse_index_flag(line: &str) -> Result<Option<IndexFlag>, IndexError> {
    // Payload-carrying flags. `-i URL` is pip's short form of `--index-url`.
    let payload_flags: &[(&[&str], fn(String) -> IndexFlag, &str)] = &[
        (
            &["--index-url", "-i"],
            IndexFlag::IndexUrl as fn(String) -> IndexFlag,
            "URL",
        ),
        (
            &["--extra-index-url"],
            IndexFlag::ExtraIndexUrl as fn(String) -> IndexFlag,
            "URL",
        ),
        (
            &["--find-links", "-f"],
            IndexFlag::FindLinks as fn(String) -> IndexFlag,
            "URL or path",
        ),
        (
            &["--no-binary"],
            IndexFlag::NoBinary as fn(String) -> IndexFlag,
            "package name(s)",
        ),
        (
            &["--only-binary"],
            IndexFlag::OnlyBinary as fn(String) -> IndexFlag,
            "package name(s)",
        ),
        (
            &["--trusted-host"],
            IndexFlag::TrustedHost as fn(String) -> IndexFlag,
            "host",
        ),
    ];
    for (aliases, ctor, what) in payload_flags {
        if let Some(rest) = strip_flag(line, aliases) {
            let payload = rest.trim();
            if payload.is_empty() {
                return Err(IndexError::ParseError {
                    url: "<requirements>".into(),
                    detail: format!("missing {what} after {}", aliases[0]),
                });
            }
            return Ok(Some(ctor(payload.to_string())));
        }
    }

    // Boolean flags — no payload allowed.
    let bare_flags: &[(&str, IndexFlag)] = &[
        ("--no-index", IndexFlag::NoIndex),
        ("--prefer-binary", IndexFlag::PreferBinary),
        ("--pre", IndexFlag::Pre),
    ];
    for (flag, value) in bare_flags {
        if line == *flag {
            return Ok(Some(value.clone()));
        }
        // Reject `--no-index=foo` / `--pre foo` — these flags take no payload.
        if let Some(rest) = line.strip_prefix(flag) {
            let next = rest.chars().next();
            if let Some(c) = next {
                if c == ' ' || c == '\t' || c == '=' {
                    return Err(IndexError::ParseError {
                        url: "<requirements>".into(),
                        detail: format!("{flag} takes no value, got: {rest:?}"),
                    });
                }
            }
        }
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Hash handling
// ---------------------------------------------------------------------------

/// True if `line` consists solely of one or more `--hash=` tokens.
/// Returns the joined hash list or `None` if other content is present.
fn parse_hash_only(line: &str) -> Option<Vec<String>> {
    let toks: Vec<&str> = line.split_whitespace().collect();
    if toks.is_empty() {
        return None;
    }
    let mut hashes = Vec::new();
    for tok in toks {
        if let Some(h) = tok.strip_prefix("--hash=") {
            hashes.push(h.to_string());
        } else {
            return None;
        }
    }
    Some(hashes)
}

/// Split inline `--hash=...` tokens off the end of a package line, returning
/// the spec head and the collected hash list.
fn split_inline_hashes(line: &str) -> (String, Vec<String>) {
    // We must be careful not to slice the line inside a marker. Hashes are
    // always at the very end (pip emits them last), so we scan tokens
    // right-to-left while they look like `--hash=...`.
    let mut head: Vec<&str> = line.split_whitespace().collect();
    let mut hashes_rev: Vec<String> = Vec::new();
    while let Some(last) = head.last() {
        if let Some(h) = last.strip_prefix("--hash=") {
            hashes_rev.push(h.to_string());
            head.pop();
        } else {
            break;
        }
    }
    hashes_rev.reverse();
    (head.join(" "), hashes_rev)
}

fn attach_hash(out: &mut [RequirementLine], hashes: Vec<String>) -> Result<(), IndexError> {
    let target = out
        .iter_mut()
        .rev()
        .find(|l| matches!(l, RequirementLine::Package(_) | RequirementLine::Editable(_)));
    let Some(target) = target else {
        return parse_err("--hash= continuation has no preceding requirement");
    };
    match target {
        RequirementLine::Package(p) => p.hashes.extend(hashes),
        RequirementLine::Editable(_) => {
            // Editable installs in pip ignore hashes; keep them on the
            // last package above the editable, if any. Since the most
            // recent context is the editable itself, drop silently rather
            // than corrupt state.
        }
        _ => unreachable!(),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Editable / package parsers
// ---------------------------------------------------------------------------

fn parse_editable(payload: &str) -> EditableSpec {
    // Editable extras: `path[extra1,extra2]` or `git+url#egg=name[extra]`.
    // pip strips extras only off the trailing `[...]` when there's no fragment.
    // We use the same simple rule and stash anything inside the last `[...]`.
    let payload = payload.trim();
    if let Some(open) = payload.rfind('[') {
        if let Some(close) = payload.rfind(']') {
            if close > open && close == payload.len() - 1 {
                let target = payload[..open].trim().to_string();
                let extras = parse_extras(&payload[open + 1..close]);
                return EditableSpec { target, extras };
            }
        }
    }
    EditableSpec {
        target: payload.to_string(),
        extras: Vec::new(),
    }
}

fn parse_package_line(line: &str) -> Result<PackageRequirement, IndexError> {
    // Split off marker `; marker` first.
    let (body, marker) = match line.find(';') {
        Some(idx) => (line[..idx].trim(), Some(line[idx + 1..].trim().to_string())),
        None => (line, None),
    };

    // Then direct URL `name @ url`. The `@` must be surrounded by whitespace
    // to distinguish it from `@` inside e.g. git refs.
    let (name_extras, direct_url) = match split_direct_url(body) {
        Some((n, u)) => (n, Some(u)),
        None => (body.to_string(), None),
    };

    // Split name vs `[extras]` vs version specifiers.
    let (name, extras, specifiers) = split_name_extras_specifiers(&name_extras)?;

    if name.is_empty() {
        return parse_err("missing package name");
    }
    validate_name(&name)?;

    Ok(PackageRequirement {
        name,
        extras,
        specifiers,
        marker,
        direct_url,
        hashes: Vec::new(),
    })
}

fn split_direct_url(body: &str) -> Option<(String, String)> {
    let bytes = body.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'@' {
            // Require surrounding whitespace on at least one side.
            let prev_ws = i == 0 || bytes[i - 1].is_ascii_whitespace();
            let next_ws = i + 1 == bytes.len() || bytes[i + 1].is_ascii_whitespace();
            if prev_ws && next_ws {
                let left = body[..i].trim().to_string();
                let right = body[i + 1..].trim().to_string();
                return Some((left, right));
            }
        }
    }
    None
}

fn split_name_extras_specifiers(
    body: &str,
) -> Result<(String, Vec<String>, Vec<String>), IndexError> {
    let body = body.trim();
    // Walk the name characters until we hit `[`, a specifier operator, or
    // whitespace. PEP 508 names are `[A-Za-z0-9._-]+`.
    let mut name_end = 0;
    for (i, c) in body.char_indices() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            name_end = i + c.len_utf8();
        } else {
            break;
        }
    }
    let name = body[..name_end].to_string();
    let rest = body[name_end..].trim_start();

    let (extras, after_extras) = if let Some(after_bracket) = rest.strip_prefix('[') {
        let close = after_bracket
            .find(']')
            .ok_or_else(|| IndexError::ParseError {
                url: "<requirements>".into(),
                detail: "unclosed '[' in extras".into(),
            })?;
        let extras = parse_extras(&after_bracket[..close]);
        (extras, after_bracket[close + 1..].trim_start())
    } else {
        (Vec::new(), rest)
    };

    let specifiers = parse_specifiers(after_extras)?;
    Ok((name, extras, specifiers))
}

fn parse_extras(src: &str) -> Vec<String> {
    src.split(',')
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect()
}

fn parse_specifiers(src: &str) -> Result<Vec<String>, IndexError> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for part in src.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        validate_specifier(s)?;
        out.push(s.to_string());
    }
    Ok(out)
}

fn validate_specifier(s: &str) -> Result<(), IndexError> {
    // Accept the PEP 440 operators: ==, ===, !=, ~=, >=, <=, >, <.
    let valid_ops = ["===", "==", "!=", "~=", ">=", "<=", ">", "<"];
    if valid_ops.iter().any(|op| s.starts_with(op)) {
        return Ok(());
    }
    Err(IndexError::ParseError {
        url: "<requirements>".into(),
        detail: format!("unrecognised version specifier '{}'", s),
    })
}

fn validate_name(name: &str) -> Result<(), IndexError> {
    // PEP 508: name = letterOrDigit ( (letterOrDigit | "-" | "_" | ".")*
    // letterOrDigit )?
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return parse_err("empty package name");
    }
    if !bytes[0].is_ascii_alphanumeric() {
        return Err(IndexError::ParseError {
            url: "<requirements>".into(),
            detail: format!("name must start with an alnum, got '{}'", name),
        });
    }
    for &b in bytes.iter() {
        if !(b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')) {
            return Err(IndexError::ParseError {
                url: "<requirements>".into(),
                detail: format!("illegal char in package name '{}'", name),
            });
        }
    }
    if !bytes.last().unwrap().is_ascii_alphanumeric() {
        return Err(IndexError::ParseError {
            url: "<requirements>".into(),
            detail: format!("name must end with an alnum, got '{}'", name),
        });
    }
    Ok(())
}

fn parse_err<T>(detail: &str) -> Result<T, IndexError> {
    Err(IndexError::ParseError {
        url: "<requirements>".into(),
        detail: detail.into(),
    })
}

/// PEP 503 normalization. Re-exported from `name_normalize` so this
/// module's existing public surface stays stable.
pub use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str) -> PackageRequirement {
        PackageRequirement {
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn parses_bare_name() {
        let r = parse_requirements_txt("requests\n").unwrap();
        assert_eq!(r, vec![RequirementLine::Package(pkg("requests"))]);
    }

    #[test]
    fn parses_pinned_version() {
        let r = parse_requirements_txt("requests==2.31.0\n").unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!("expected package");
        };
        assert_eq!(p.name, "requests");
        assert_eq!(p.specifiers, vec!["==2.31.0"]);
    }

    #[test]
    fn parses_compound_specifiers() {
        let r = parse_requirements_txt("django>=4.2,<5\n").unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.specifiers, vec![">=4.2", "<5"]);
    }

    #[test]
    fn parses_extras() {
        let r = parse_requirements_txt("uvicorn[standard,http]==0.30.0\n").unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.name, "uvicorn");
        assert_eq!(p.extras, vec!["standard", "http"]);
        assert_eq!(p.specifiers, vec!["==0.30.0"]);
    }

    #[test]
    fn parses_marker() {
        let r =
            parse_requirements_txt("tomli==2.0.1 ; python_version < \"3.11\"\n").unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.marker.as_deref(), Some("python_version < \"3.11\""));
    }

    #[test]
    fn parses_direct_url_git() {
        let r =
            parse_requirements_txt("mypkg @ git+https://example.com/mypkg.git@v1.0\n").unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.name, "mypkg");
        assert_eq!(p.direct_url.as_deref(), Some("git+https://example.com/mypkg.git@v1.0"));
        assert!(p.specifiers.is_empty());
    }

    #[test]
    fn parses_direct_url_file() {
        let r = parse_requirements_txt("local @ file:///tmp/wheels/local.whl\n").unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.direct_url.as_deref(), Some("file:///tmp/wheels/local.whl"));
    }

    #[test]
    fn parses_inline_hashes() {
        let r = parse_requirements_txt(
            "requests==2.31.0 --hash=sha256:aaaa --hash=sha256:bbbb\n",
        )
        .unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.hashes, vec!["sha256:aaaa", "sha256:bbbb"]);
    }

    #[test]
    fn parses_continuation_hashes() {
        let src = "requests==2.31.0 \\\n    --hash=sha256:aaaa \\\n    --hash=sha256:bbbb\n";
        let r = parse_requirements_txt(src).unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.hashes, vec!["sha256:aaaa", "sha256:bbbb"]);
    }

    #[test]
    fn parses_hash_on_own_line_attaches_to_prior() {
        let src = "requests==2.31.0\n    --hash=sha256:zzzz\n";
        let r = parse_requirements_txt(src).unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(p.hashes, vec!["sha256:zzzz"]);
    }

    #[test]
    fn parses_include_directive() {
        let r = parse_requirements_txt("-r common.txt\n").unwrap();
        assert_eq!(r, vec![RequirementLine::Include("common.txt".into())]);
        let r =
            parse_requirements_txt("--requirement=other/base.txt\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::Include("other/base.txt".into())]
        );
    }

    #[test]
    fn parses_constraint_directive() {
        let r = parse_requirements_txt("-c constraints.txt\n").unwrap();
        assert_eq!(r, vec![RequirementLine::Constraint("constraints.txt".into())]);
    }

    #[test]
    fn parses_editable_directive() {
        let r = parse_requirements_txt("-e .\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::Editable(EditableSpec {
                target: ".".into(),
                extras: Vec::new(),
            })]
        );
    }

    #[test]
    fn parses_editable_with_extras() {
        let r = parse_requirements_txt("-e .[dev,test]\n").unwrap();
        let RequirementLine::Editable(e) = &r[0] else {
            panic!()
        };
        assert_eq!(e.target, ".");
        assert_eq!(e.extras, vec!["dev", "test"]);
    }

    #[test]
    fn parses_editable_git_url() {
        let r = parse_requirements_txt(
            "--editable git+https://example.com/pkg.git@main\n",
        )
        .unwrap();
        let RequirementLine::Editable(e) = &r[0] else {
            panic!()
        };
        assert_eq!(e.target, "git+https://example.com/pkg.git@main");
    }

    #[test]
    fn strips_comments() {
        let src = "# header comment\nrequests # trailing\n  # alone\nclick\n";
        let r = parse_requirements_txt(src).unwrap();
        assert_eq!(r.len(), 2);
        let RequirementLine::Package(p0) = &r[0] else {
            panic!()
        };
        assert_eq!(p0.name, "requests");
    }

    #[test]
    fn comment_inside_marker_quotes_is_preserved() {
        let src = "pkg==1.0 ; sys_platform == \"linux#weird\"\n";
        let r = parse_requirements_txt(src).unwrap();
        let RequirementLine::Package(p) = &r[0] else {
            panic!()
        };
        assert_eq!(
            p.marker.as_deref(),
            Some("sys_platform == \"linux#weird\"")
        );
    }

    #[test]
    fn skips_blank_lines() {
        let src = "\n\nrequests\n\n\nclick\n";
        let r = parse_requirements_txt(src).unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn unknown_flag_is_preserved() {
        // `--config-settings` is not a known pip-compat directive at the
        // requirements.txt layer; the parser must keep it verbatim.
        let r = parse_requirements_txt("--config-settings foo=bar\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::Unknown("--config-settings foo=bar".into())]
        );
    }

    // ----- Tick 71: typed pip index/binary flags --------------------------

    #[test]
    fn parses_index_url_flag() {
        let r = parse_requirements_txt("--index-url https://example.com/simple/\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::IndexUrl(
                "https://example.com/simple/".into()
            ))]
        );
    }

    #[test]
    fn parses_index_url_short_form() {
        let r = parse_requirements_txt("-i https://example.com/simple/\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::IndexUrl(
                "https://example.com/simple/".into()
            ))]
        );
    }

    #[test]
    fn parses_index_url_equals_form() {
        let r = parse_requirements_txt("--index-url=https://example.com/simple/\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::IndexUrl(
                "https://example.com/simple/".into()
            ))]
        );
    }

    #[test]
    fn parses_extra_index_url_flag() {
        let r =
            parse_requirements_txt("--extra-index-url https://mirror.example.com/\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::ExtraIndexUrl(
                "https://mirror.example.com/".into()
            ))]
        );
    }

    #[test]
    fn parses_find_links_flag() {
        let r = parse_requirements_txt("--find-links /tmp/wheels\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::FindLinks(
                "/tmp/wheels".into()
            ))]
        );
        let r = parse_requirements_txt("-f https://wheels.example.com/\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::FindLinks(
                "https://wheels.example.com/".into()
            ))]
        );
    }

    #[test]
    fn parses_no_index_bare_flag() {
        let r = parse_requirements_txt("--no-index\n").unwrap();
        assert_eq!(r, vec![RequirementLine::IndexFlag(IndexFlag::NoIndex)]);
    }

    #[test]
    fn parses_pre_and_prefer_binary_bare_flags() {
        let r = parse_requirements_txt("--pre\n--prefer-binary\n").unwrap();
        assert_eq!(
            r,
            vec![
                RequirementLine::IndexFlag(IndexFlag::Pre),
                RequirementLine::IndexFlag(IndexFlag::PreferBinary),
            ]
        );
    }

    #[test]
    fn parses_no_binary_with_package_list() {
        let r = parse_requirements_txt("--no-binary pillow,numpy\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::NoBinary(
                "pillow,numpy".into()
            ))]
        );
    }

    #[test]
    fn parses_no_binary_all_sentinel() {
        let r = parse_requirements_txt("--no-binary :all:\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::NoBinary(":all:".into()))]
        );
    }

    #[test]
    fn parses_only_binary_package_list() {
        let r = parse_requirements_txt("--only-binary pillow\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::OnlyBinary(
                "pillow".into()
            ))]
        );
    }

    #[test]
    fn parses_trusted_host_flag() {
        let r = parse_requirements_txt("--trusted-host mirror.example.com\n").unwrap();
        assert_eq!(
            r,
            vec![RequirementLine::IndexFlag(IndexFlag::TrustedHost(
                "mirror.example.com".into()
            ))]
        );
    }

    #[test]
    fn payload_flag_without_value_errors() {
        let err = parse_requirements_txt("--index-url\n").unwrap_err();
        match err {
            IndexError::ParseError { detail, .. } => {
                assert!(detail.contains("missing URL"), "got: {detail}");
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn boolean_flag_with_value_errors() {
        let err = parse_requirements_txt("--no-index=true\n").unwrap_err();
        match err {
            IndexError::ParseError { detail, .. } => {
                assert!(detail.contains("takes no value"), "got: {detail}");
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn index_flags_interleave_with_requirements() {
        let src = "\
--index-url https://example.com/simple/
--no-binary pillow
click==8.1.7
--pre
requests
";
        let r = parse_requirements_txt(src).unwrap();
        assert!(matches!(
            r[0],
            RequirementLine::IndexFlag(IndexFlag::IndexUrl(_))
        ));
        assert!(matches!(
            r[1],
            RequirementLine::IndexFlag(IndexFlag::NoBinary(_))
        ));
        assert!(matches!(r[2], RequirementLine::Package(_)));
        assert!(matches!(r[3], RequirementLine::IndexFlag(IndexFlag::Pre)));
        assert!(matches!(r[4], RequirementLine::Package(_)));
    }

    #[test]
    fn rejects_unknown_specifier_op() {
        let err = parse_requirements_txt("pkg=1.0\n").unwrap_err();
        match err {
            IndexError::ParseError { detail, .. } => {
                assert!(detail.contains("unrecognised version specifier"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn rejects_bad_name() {
        let err = parse_requirements_txt("@bad==1\n").unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn rejects_dangling_hash() {
        let err = parse_requirements_txt("--hash=sha256:aaaa\n").unwrap_err();
        assert!(matches!(err, IndexError::ParseError { .. }));
    }

    #[test]
    fn handles_crlf_line_endings() {
        let r = parse_requirements_txt("requests==2.31.0\r\nclick\r\n").unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn pep503_normalize_collapses_separators() {
        assert_eq!(pep503_normalize("My_Pkg"), "my-pkg");
        assert_eq!(pep503_normalize("foo..bar"), "foo-bar");
        assert_eq!(pep503_normalize("foo-_-bar"), "foo-bar");
    }

    #[test]
    fn round_trips_simple_export_shape() {
        // Approximate inverse of Tick 41 exporter: `name==version` plus
        // optional marker. Parser must accept what the exporter emits.
        let src = "\
click==8.1.7
requests==2.31.0 ; python_version >= \"3.8\"
";
        let r = parse_requirements_txt(src).unwrap();
        assert_eq!(r.len(), 2);
        let RequirementLine::Package(p0) = &r[0] else {
            panic!()
        };
        let RequirementLine::Package(p1) = &r[1] else {
            panic!()
        };
        assert_eq!(p0.name, "click");
        assert_eq!(p0.specifiers, vec!["==8.1.7"]);
        assert_eq!(p1.name, "requests");
        assert_eq!(p1.marker.as_deref(), Some("python_version >= \"3.8\""));
    }

    #[test]
    fn parses_complex_mixed_file() {
        let src = "\
# Generated by mamba
-r base.txt
-c constraints.txt
--index-url https://pypi.org/simple/

click==8.1.7
requests[security]>=2.30,<3 ; python_version >= \"3.8\" \\
    --hash=sha256:aaaa \\
    --hash=sha256:bbbb
mypkg @ git+https://example.com/m.git@v1.0
-e .[dev]
";
        let r = parse_requirements_txt(src).unwrap();
        assert_eq!(r.len(), 7);
        assert!(matches!(r[0], RequirementLine::Include(_)));
        assert!(matches!(r[1], RequirementLine::Constraint(_)));
        assert!(matches!(
            r[2],
            RequirementLine::IndexFlag(IndexFlag::IndexUrl(_))
        ));
        let RequirementLine::Package(p3) = &r[3] else {
            panic!()
        };
        assert_eq!(p3.name, "click");
        let RequirementLine::Package(p4) = &r[4] else {
            panic!()
        };
        assert_eq!(p4.extras, vec!["security"]);
        assert_eq!(p4.specifiers, vec![">=2.30", "<3"]);
        assert_eq!(p4.hashes.len(), 2);
        let RequirementLine::Package(p5) = &r[5] else {
            panic!()
        };
        assert_eq!(p5.direct_url.as_deref(), Some("git+https://example.com/m.git@v1.0"));
        let RequirementLine::Editable(e6) = &r[6] else {
            panic!()
        };
        assert_eq!(e6.extras, vec!["dev"]);
    }
}
