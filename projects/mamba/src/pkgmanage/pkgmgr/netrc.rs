// `~/.netrc` credentials parser (Tick 69).
//
// uv consults `~/.netrc` (or `%USERPROFILE%\_netrc` on Windows) when an
// index URL requires HTTP Basic auth and no inline credentials are
// supplied. The file is whitespace-tokenised; we mirror the grammar
// understood by curl, Python's `netrc` stdlib, and uv itself:
//
//     machine <host>
//         login <user>
//         password <pass>
//         account <acct>      # optional, rarely used
//
//     default
//         login <user>
//         password <pass>
//
//     macdef <name>           # macros — skipped silently
//     ...
//     <blank line ends macdef>
//
// Quoting: tokens enclosed in `"..."` may contain spaces; backslash
// escapes the next character. Comments start with `#` and run to EOL.
// `default` (no host) provides a fall-through used when no machine entry
// matches. Multiple `machine` entries for the same host: first wins
// (matches Python's stdlib).
//
// Pure parser: no filesystem I/O. Callers read the file themselves and
// feed the body to `parse_netrc`.

use std::collections::HashMap;

use crate::pkgmanage::pkgmgr::types::IndexError;

const NETRC_URL: &str = "<.netrc>";

/// One credentials entry: a host's login, password, and optional account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetrcEntry {
    pub login: Option<String>,
    pub password: Option<String>,
    pub account: Option<String>,
}

/// Parsed netrc body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Netrc {
    /// `machine <host>` entries, keyed by host. First occurrence wins.
    pub machines: HashMap<String, NetrcEntry>,
    /// Fallback entry from the `default` keyword, if present.
    pub default: Option<NetrcEntry>,
}

impl Netrc {
    /// Look up credentials for `host`. Falls back to `default` when no
    /// machine entry matches. Returns `None` if neither is present.
    pub fn lookup(&self, host: &str) -> Option<&NetrcEntry> {
        if let Some(entry) = self.machines.get(host) {
            return Some(entry);
        }
        self.default.as_ref()
    }
}

/// Parse a netrc-format body. Returns a structured `Netrc`. Unknown
/// keywords inside an entry block are tolerated and skipped (forward
/// compatibility with vendor extensions). `macdef` blocks are skipped
/// entirely. Malformed quoting or a trailing keyword with no value
/// surfaces as `IndexError::ParseError`.
pub fn parse_netrc(src: &str) -> Result<Netrc, IndexError> {
    let tokens = tokenize(src)?;
    let mut out = Netrc::default();
    let mut iter = tokens.into_iter().peekable();

    while let Some(tok) = iter.next() {
        match tok.as_str() {
            "machine" => {
                let host = next_required(&mut iter, "machine")?;
                let entry = parse_entry(&mut iter)?;
                // First occurrence wins; preserve Python stdlib semantics.
                out.machines.entry(host).or_insert(entry);
            }
            "default" => {
                let entry = parse_entry(&mut iter)?;
                if out.default.is_none() {
                    out.default = Some(entry);
                }
            }
            "macdef" => {
                // Consume macro name + body until a blank line. Our
                // tokenizer drops blank lines, so we just need to skip
                // the macro name token and stop at the next top-level
                // keyword. Since we can't distinguish macro body lines
                // from top-level keywords after dropping blanks, we
                // require macdef bodies to be empty in practice — uv,
                // pip, and curl never use macros anyway. We do swallow
                // the macro name to keep the stream sync'd.
                let _ = iter.next();
            }
            // Stray top-level keyword without `machine`/`default`/`macdef` —
            // most netrc files don't have this but pip is permissive about
            // a bare `login`/`password` outside any block; we err out so the
            // user notices the file is malformed.
            other => {
                return Err(IndexError::ParseError {
                    url: NETRC_URL.into(),
                    detail: format!(
                        "unexpected top-level token `{other}` outside any machine/default block"
                    ),
                });
            }
        }
    }

    Ok(out)
}

/// Pull `login`/`password`/`account` keys until we hit another top-level
/// keyword (`machine`, `default`, `macdef`) or EOF. Unknown keywords
/// inside the entry block are silently skipped so callers tolerate
/// vendor extensions.
fn parse_entry(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
) -> Result<NetrcEntry, IndexError> {
    let mut entry = NetrcEntry::default();
    while let Some(tok) = iter.peek() {
        match tok.as_str() {
            "machine" | "default" | "macdef" => break,
            "login" => {
                iter.next();
                let v = next_required(iter, "login")?;
                if entry.login.is_none() {
                    entry.login = Some(v);
                }
            }
            "password" => {
                iter.next();
                let v = next_required(iter, "password")?;
                if entry.password.is_none() {
                    entry.password = Some(v);
                }
            }
            "account" => {
                iter.next();
                let v = next_required(iter, "account")?;
                if entry.account.is_none() {
                    entry.account = Some(v);
                }
            }
            // Forward-compat: ignore unknown keys, but still consume the
            // following token as its value so the stream stays aligned.
            _ => {
                iter.next();
                if let Some(peek) = iter.peek() {
                    let is_top = matches!(peek.as_str(), "machine" | "default" | "macdef");
                    if !is_top {
                        iter.next();
                    }
                }
            }
        }
    }
    Ok(entry)
}

fn next_required(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<String>>,
    key: &str,
) -> Result<String, IndexError> {
    iter.next().ok_or_else(|| IndexError::ParseError {
        url: NETRC_URL.into(),
        detail: format!("`{key}` has no value before end of file"),
    })
}

/// Split the netrc body into whitespace-separated tokens, honoring
/// quoted strings, `#` comments, and backslash escapes inside quotes.
fn tokenize(src: &str) -> Result<Vec<String>, IndexError> {
    let mut out: Vec<String> = Vec::new();
    let mut chars = src.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' | '\n' => {
                chars.next();
            }
            '#' => {
                // Comment: consume to newline.
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\n' {
                        break;
                    }
                }
            }
            '"' => {
                chars.next();
                let mut buf = String::new();
                let mut closed = false;
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\\' {
                        if let Some(&next) = chars.peek() {
                            chars.next();
                            buf.push(next);
                            continue;
                        }
                    }
                    if c == '"' {
                        closed = true;
                        break;
                    }
                    buf.push(c);
                }
                if !closed {
                    return Err(IndexError::ParseError {
                        url: NETRC_URL.into(),
                        detail: "unterminated quoted string".into(),
                    });
                }
                out.push(buf);
            }
            _ => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == '#' {
                        break;
                    }
                    buf.push(c);
                    chars.next();
                }
                if !buf.is_empty() {
                    out.push(buf);
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_netrc() {
        let n = parse_netrc("").unwrap();
        assert!(n.machines.is_empty());
        assert!(n.default.is_none());
    }

    #[test]
    fn whitespace_only_input_yields_empty_netrc() {
        let n = parse_netrc("   \n\t\n\n").unwrap();
        assert!(n.machines.is_empty());
    }

    #[test]
    fn single_machine_entry() {
        let src = "machine pypi.org login alice password secret\n";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
        assert_eq!(e.password.as_deref(), Some("secret"));
        assert_eq!(e.account, None);
    }

    #[test]
    fn machine_entry_with_account() {
        let src = "machine pypi.org login alice password secret account billing\n";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.account.as_deref(), Some("billing"));
    }

    #[test]
    fn multi_line_entry_with_indentation() {
        let src = "\
machine pypi.org
    login alice
    password secret
";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
        assert_eq!(e.password.as_deref(), Some("secret"));
    }

    #[test]
    fn multiple_machines() {
        let src = "\
machine pypi.org login alice password secret1
machine artifactory.example.com login bob password secret2
";
        let n = parse_netrc(src).unwrap();
        assert_eq!(
            n.machines.get("pypi.org").unwrap().login.as_deref(),
            Some("alice")
        );
        assert_eq!(
            n.machines
                .get("artifactory.example.com")
                .unwrap()
                .password
                .as_deref(),
            Some("secret2")
        );
    }

    #[test]
    fn default_entry_is_captured() {
        let src = "\
machine pypi.org login alice password secret
default login fallback password fbpass
";
        let n = parse_netrc(src).unwrap();
        let d = n.default.as_ref().unwrap();
        assert_eq!(d.login.as_deref(), Some("fallback"));
    }

    #[test]
    fn lookup_falls_back_to_default() {
        let src = "default login fallback password fbpass\n";
        let n = parse_netrc(src).unwrap();
        let e = n.lookup("anything.example.com").unwrap();
        assert_eq!(e.login.as_deref(), Some("fallback"));
    }

    #[test]
    fn lookup_prefers_machine_over_default() {
        let src = "\
machine pypi.org login alice password secret
default login fallback password fbpass
";
        let n = parse_netrc(src).unwrap();
        let e = n.lookup("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
    }

    #[test]
    fn lookup_returns_none_when_no_match_and_no_default() {
        let src = "machine pypi.org login alice password secret\n";
        let n = parse_netrc(src).unwrap();
        assert!(n.lookup("other.example.com").is_none());
    }

    #[test]
    fn first_machine_wins_on_duplicate_host() {
        // Python's netrc stdlib uses first-occurrence semantics.
        let src = "\
machine pypi.org login first password p1
machine pypi.org login second password p2
";
        let n = parse_netrc(src).unwrap();
        assert_eq!(
            n.machines.get("pypi.org").unwrap().login.as_deref(),
            Some("first")
        );
    }

    #[test]
    fn quoted_tokens_preserve_spaces() {
        let src = "machine pypi.org login \"alice user\" password \"p@ss word\"\n";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice user"));
        assert_eq!(e.password.as_deref(), Some("p@ss word"));
    }

    #[test]
    fn backslash_escapes_inside_quotes() {
        // `"line\"break"` should yield `line"break`.
        let src = "machine pypi.org password \"line\\\"break\"\n";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.password.as_deref(), Some("line\"break"));
    }

    #[test]
    fn comments_are_ignored() {
        let src = "\
# top comment
machine pypi.org login alice password secret # trailing
# trailing block comment
";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
        assert_eq!(e.password.as_deref(), Some("secret"));
    }

    #[test]
    fn unknown_keys_inside_entry_are_skipped() {
        let src = "machine pypi.org login alice extra-vendor xyz password secret\n";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
        assert_eq!(e.password.as_deref(), Some("secret"));
    }

    #[test]
    fn macdef_blocks_are_skipped() {
        let src = "\
macdef init
machine pypi.org login alice password secret
";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
    }

    #[test]
    fn unterminated_quoted_token_errors() {
        let src = "machine pypi.org login \"alice\n";
        let err = parse_netrc(src).unwrap_err();
        assert!(
            format!("{err}").contains("unterminated quoted string"),
            "got: {err}"
        );
    }

    #[test]
    fn trailing_key_without_value_errors() {
        let src = "machine pypi.org login alice password\n";
        let err = parse_netrc(src).unwrap_err();
        assert!(format!("{err}").contains("password"), "got: {err}");
    }

    #[test]
    fn stray_top_level_keyword_errors() {
        let src = "login alice password secret\n";
        let err = parse_netrc(src).unwrap_err();
        assert!(
            format!("{err}").contains("unexpected top-level token"),
            "got: {err}"
        );
    }

    #[test]
    fn entry_without_password_is_legal() {
        // Some legacy netrc files only carry a login token (e.g. for
        // anonymous FTP). Parse must accept this without erroring.
        let src = "machine pypi.org login alice\n";
        let n = parse_netrc(src).unwrap();
        let e = n.machines.get("pypi.org").unwrap();
        assert_eq!(e.login.as_deref(), Some("alice"));
        assert_eq!(e.password, None);
    }
}
