// requirements_options.rs — pip-compatible file-scope flag parser.
//
// `pip install -r requirements.txt` recognises a handful of options
// that apply to the whole file rather than to one requirement. uv
// honours the same set when it ingests pip-flavored input. Mamba's
// existing requirements_parse handles per-line `name==ver` /
// `--hash=` and requirements_loader handles nested `-r/-c` includes;
// this module fills the remaining gap — the file-prelude flags.
//
// Recognised:
//
//   -i URL, --index-url URL       Primary PEP 503 simple index (single).
//   --extra-index-url URL         Additional simple index (repeatable).
//   --find-links URL_OR_DIR       Legacy `<a>`-listing page (repeatable).
//   --no-index                    Don't query PyPI default at all.
//   --trusted-host HOST[:PORT]    Skip cert validation (repeatable).
//   --pre                         Allow pre-releases.
//   --require-hashes              Mandate --hash on every requirement.
//
// `--<opt>=value` and `--<opt> value` are both accepted. Quoted
// values are unwrapped (`'…'` and `"…"`). Lines starting with `#`
// are comments. Blank lines skipped. Unknown options pass through to
// the remaining-lines vec so per-line parsers can deal with them (or
// flag them).

use crate::pkgmanage::pkgmgr::types::IndexError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GlobalOptions {
    /// `-i URL` / `--index-url URL`. None means "use default".
    pub index_url: Option<String>,
    /// `--extra-index-url URL`, repeatable.
    pub extra_index_urls: Vec<String>,
    /// `--find-links URL_OR_DIR`, repeatable.
    pub find_links: Vec<String>,
    /// `--no-index` — refuse to query the default PyPI index.
    pub no_index: bool,
    /// `--trusted-host HOST[:PORT]`, repeatable.
    pub trusted_hosts: Vec<String>,
    /// `--pre` — allow pre-release versions during resolution.
    pub pre: bool,
    /// `--require-hashes` — every requirement must carry --hash.
    pub require_hashes: bool,
}

/// Walk a requirements.txt body. Returns the parsed global options
/// plus all non-global lines (with surrounding whitespace trimmed,
/// blanks and comments dropped) for downstream per-requirement
/// parsing.
pub fn extract_global_options(src: &str) -> Result<(GlobalOptions, Vec<String>), IndexError> {
    let mut opts = GlobalOptions::default();
    let mut rest: Vec<String> = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let line_no = lineno + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Strip trailing comment (`# …` with leading whitespace).
        let stripped = strip_trailing_comment(line);
        if stripped.is_empty() {
            continue;
        }
        let (name, value) = split_option(stripped);
        match name {
            "-i" | "--index-url" => {
                let v = require_value(name, value, line_no)?;
                opts.index_url = Some(v);
            }
            "--extra-index-url" => {
                let v = require_value(name, value, line_no)?;
                opts.extra_index_urls.push(v);
            }
            "--find-links" | "-f" => {
                let v = require_value(name, value, line_no)?;
                opts.find_links.push(v);
            }
            "--trusted-host" => {
                let v = require_value(name, value, line_no)?;
                opts.trusted_hosts.push(v);
            }
            "--no-index" => {
                reject_value(name, value, line_no)?;
                opts.no_index = true;
            }
            "--pre" => {
                reject_value(name, value, line_no)?;
                opts.pre = true;
            }
            "--require-hashes" => {
                reject_value(name, value, line_no)?;
                opts.require_hashes = true;
            }
            // Everything else (package specifier, `-r/-c/-e`, or a
            // truly unknown option) goes to the next layer.
            _ => rest.push(stripped.to_string()),
        }
    }
    Ok((opts, rest))
}

/// Strip an inline `# …` comment. A `#` is treated as a comment iff
/// it follows whitespace (so `pkg#egg=foo` stays intact).
fn strip_trailing_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' && (i == 0 || matches!(bytes[i - 1], b' ' | b'\t')) {
            return line[..i].trim_end();
        }
        i += 1;
    }
    line
}

/// Split a recognised-option line into `(name, optional value)`.
/// Supports both `--opt=value` and `--opt value` (and `-i value`).
fn split_option(line: &str) -> (&str, Option<String>) {
    // Inline `--opt=value` form.
    if let Some((name, val)) = line.split_once('=') {
        if name.starts_with('-') && !name.contains(char::is_whitespace) {
            return (name, Some(unquote(val.trim()).to_string()));
        }
    }
    // Space-separated `--opt value` form.
    let mut parts = line.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("");
    let value = parts.next().map(|s| unquote(s.trim()).to_string());
    if value.as_deref().map(str::is_empty).unwrap_or(false) {
        return (name, None);
    }
    (name, value)
}

fn unquote(s: &str) -> &str {
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        let (first, last) = (bytes[0], bytes[bytes.len() - 1]);
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

fn require_value(name: &str, value: Option<String>, line_no: usize) -> Result<String, IndexError> {
    match value {
        Some(v) if !v.is_empty() => Ok(v),
        _ => Err(pe(&format!(
            "line {line_no}: option {name} requires a value"
        ))),
    }
}

fn reject_value(name: &str, value: Option<String>, line_no: usize) -> Result<(), IndexError> {
    if value.is_some() {
        return Err(pe(&format!("line {line_no}: option {name} takes no value")));
    }
    Ok(())
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "requirements.txt".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt_only(src: &str) -> GlobalOptions {
        extract_global_options(src).unwrap().0
    }

    #[test]
    fn parses_index_url_long_form() {
        let opts = opt_only("--index-url https://idx/simple/\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/simple/"));
    }

    #[test]
    fn parses_index_url_short_form() {
        let opts = opt_only("-i https://idx/simple/\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/simple/"));
    }

    #[test]
    fn parses_index_url_equals_form() {
        let opts = opt_only("--index-url=https://idx/simple/\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/simple/"));
    }

    #[test]
    fn parses_repeated_extra_index_url() {
        let src = "\
--extra-index-url https://a/
--extra-index-url=https://b/
";
        let opts = opt_only(src);
        assert_eq!(opts.extra_index_urls, vec!["https://a/", "https://b/"]);
    }

    #[test]
    fn parses_find_links_long_and_short() {
        let src = "\
--find-links https://x/
-f /local/wheels
";
        let opts = opt_only(src);
        assert_eq!(opts.find_links, vec!["https://x/", "/local/wheels"]);
    }

    #[test]
    fn parses_trusted_host_with_port() {
        let opts = opt_only("--trusted-host idx.example.com:8443\n");
        assert_eq!(opts.trusted_hosts, vec!["idx.example.com:8443"]);
    }

    #[test]
    fn parses_no_index_pre_require_hashes() {
        let src = "--no-index\n--pre\n--require-hashes\n";
        let opts = opt_only(src);
        assert!(opts.no_index);
        assert!(opts.pre);
        assert!(opts.require_hashes);
    }

    #[test]
    fn double_quoted_value_unwrapped() {
        let opts = opt_only("--index-url \"https://idx/simple/\"\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/simple/"));
    }

    #[test]
    fn single_quoted_value_unwrapped() {
        let opts = opt_only("--index-url='https://idx/simple/'\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/simple/"));
    }

    #[test]
    fn blanks_and_full_line_comments_skipped() {
        let src = "\
# this is a comment

--index-url https://idx/

# trailing comment
";
        let opts = opt_only(src);
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/"));
    }

    #[test]
    fn inline_comment_stripped_when_preceded_by_whitespace() {
        let opts = opt_only("--index-url https://idx/  # the index\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/"));
    }

    #[test]
    fn hash_inside_url_segment_preserved() {
        // `pkg#egg=foo` is a real pip pattern. The `#` does NOT start
        // a comment because no whitespace precedes it.
        let (_opts, rest) = extract_global_options("foo @ https://x/y#egg=foo\n").unwrap();
        assert_eq!(rest, vec!["foo @ https://x/y#egg=foo"]);
    }

    #[test]
    fn unknown_options_passed_through_to_rest() {
        let (opts, rest) =
            extract_global_options("--index-url https://idx/\n--brand-new-flag\nrequests==2.31\n")
                .unwrap();
        assert_eq!(opts.index_url.as_deref(), Some("https://idx/"));
        assert_eq!(rest, vec!["--brand-new-flag", "requests==2.31"]);
    }

    #[test]
    fn nested_includes_passed_through_to_rest() {
        // -r / -c handled by requirements_loader, not us.
        let (_, rest) = extract_global_options("-r other.txt\n-c con.txt\n").unwrap();
        assert_eq!(rest, vec!["-r other.txt", "-c con.txt"]);
    }

    #[test]
    fn editable_passed_through_to_rest() {
        let (_, rest) = extract_global_options("-e git+https://x/y.git#egg=y\n").unwrap();
        assert_eq!(rest, vec!["-e git+https://x/y.git#egg=y"]);
    }

    #[test]
    fn rejects_option_missing_value() {
        let err = extract_global_options("--index-url\n").unwrap_err();
        assert!(err.to_string().contains("requires a value"));
    }

    #[test]
    fn rejects_value_passed_to_boolean_flag() {
        let err = extract_global_options("--pre yes\n").unwrap_err();
        assert!(err.to_string().contains("takes no value"));
    }

    #[test]
    fn rejects_equals_value_passed_to_boolean_flag() {
        let err = extract_global_options("--no-index=true\n").unwrap_err();
        assert!(err.to_string().contains("takes no value"));
    }

    #[test]
    fn error_messages_include_one_based_line_number() {
        let src = "# comment\n\n--index-url\n";
        let err = extract_global_options(src).unwrap_err();
        assert!(err.to_string().contains("line 3"), "got {err}");
    }

    #[test]
    fn full_realistic_prelude() {
        let src = "\
# pinned by uv pip compile
--index-url https://pypi.org/simple/
--extra-index-url https://internal.example.com/simple/
--trusted-host internal.example.com
--require-hashes

requests==2.31.0 \\
    --hash=sha256:0000000000000000000000000000000000000000000000000000000000000000
";
        let (opts, rest) = extract_global_options(src).unwrap();
        assert_eq!(opts.index_url.as_deref(), Some("https://pypi.org/simple/"));
        assert_eq!(
            opts.extra_index_urls,
            vec!["https://internal.example.com/simple/"]
        );
        assert_eq!(opts.trusted_hosts, vec!["internal.example.com"]);
        assert!(opts.require_hashes);
        assert!(!opts.pre);
        assert!(!opts.no_index);
        // Both the spec line and its continuation hash flag survive.
        assert!(rest.iter().any(|l| l.starts_with("requests==")));
        assert!(rest.iter().any(|l| l.contains("--hash=sha256")));
    }

    #[test]
    fn empty_input_yields_default_options_and_empty_rest() {
        let (opts, rest) = extract_global_options("").unwrap();
        assert_eq!(opts, GlobalOptions::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn whitespace_only_input_yields_default_options() {
        let (opts, rest) = extract_global_options("   \n\t\n   ").unwrap();
        assert_eq!(opts, GlobalOptions::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn duplicate_index_url_last_wins() {
        // pip itself silently overrides — match that behavior so users
        // can layer requirements files predictably.
        let opts = opt_only("--index-url https://a/\n--index-url https://b/\n");
        assert_eq!(opts.index_url.as_deref(), Some("https://b/"));
    }
}
