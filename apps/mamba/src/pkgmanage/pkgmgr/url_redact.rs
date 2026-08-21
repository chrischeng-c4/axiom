// url_redact.rs — strip / redact userinfo from URLs.
//
// Two recurring needs across the package manager:
//
//   * `redact_credentials(url)` — for human-facing surfaces (logs,
//     error messages, progress bars). Replaces the userinfo segment
//     with `****` so the operator can see "yes, creds were involved"
//     without leaking secrets.
//
//   * `strip_credentials(url)` — for machine-facing keys (HTTP cache
//     index, lockfile source ref, dedup hash). Drops the userinfo
//     entirely so `https://user:pw@pypi.org/simple/x/` and
//     `https://pypi.org/simple/x/` hash to the same key.
//
// RFC 3986 §3.2.1 userinfo grammar:
//   authority = [ userinfo "@" ] host [ ":" port ]
//   userinfo  = *( unreserved / pct-encoded / sub-delims / ":" )
//
// We rely on the `userinfo@` shape — first `@` between scheme://
// and the next `/?#` separator is the credentials terminator. URLs
// without an `@` in their authority pass through unchanged.
//
// Inputs that don't look like a `scheme://…` URL (e.g. plain file
// paths, git refs) are passed through verbatim; callers can log /
// hash them as-is.

/// Human-facing redaction: replace `user:pass@` with `****@`. The
/// scheme, host, port, and path are preserved exactly.
pub fn redact_credentials(url: &str) -> String {
    rewrite(url, |_userinfo| Some("****".to_string()))
}

/// Machine-facing strip: remove `user:pass@` entirely so the
/// resulting URL hashes the same with or without credentials.
pub fn strip_credentials(url: &str) -> String {
    rewrite(url, |_userinfo| None)
}

/// Shared scaffolding for both operations. `replace` receives the raw
/// userinfo segment (without the trailing `@`) and returns either
/// `Some(new_userinfo)` to substitute, or `None` to drop the
/// userinfo+`@` pair entirely.
fn rewrite(url: &str, replace: impl Fn(&str) -> Option<String>) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };
    let after_scheme_idx = scheme_end + 3;
    let after_scheme = &url[after_scheme_idx..];

    // The authority ends at the first `/`, `?`, or `#`.
    let authority_end_rel = after_scheme
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(after_scheme.len());
    let authority = &after_scheme[..authority_end_rel];

    // Only the LAST `@` in the authority separates userinfo from
    // host (RFC 3986 allows `@` inside pct-encoded userinfo, but in
    // practice we want to be permissive: take everything up to the
    // last `@` as userinfo).
    let Some(at_rel) = authority.rfind('@') else {
        return url.to_string();
    };

    let userinfo = &authority[..at_rel];
    let rest_of_authority = &authority[at_rel + 1..];
    let tail = &after_scheme[authority_end_rel..];
    let scheme_part = &url[..after_scheme_idx];

    match replace(userinfo) {
        Some(new_userinfo) => {
            format!("{scheme_part}{new_userinfo}@{rest_of_authority}{tail}")
        }
        None => format!("{scheme_part}{rest_of_authority}{tail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_user_pass() {
        assert_eq!(
            redact_credentials("https://user:pw@host/path"),
            "https://****@host/path"
        );
    }

    #[test]
    fn redact_user_only() {
        // GitHub PAT shape: token in user field, no `:pass`.
        assert_eq!(
            redact_credentials("https://ghp_abc123@host/path"),
            "https://****@host/path"
        );
    }

    #[test]
    fn redact_preserves_port() {
        assert_eq!(
            redact_credentials("https://u:p@host:8080/path"),
            "https://****@host:8080/path"
        );
    }

    #[test]
    fn redact_preserves_query_and_fragment() {
        assert_eq!(
            redact_credentials("https://u:p@host/path?q=1&r=2#frag"),
            "https://****@host/path?q=1&r=2#frag"
        );
    }

    #[test]
    fn redact_no_credentials_passes_through() {
        assert_eq!(
            redact_credentials("https://pypi.org/simple/x/"),
            "https://pypi.org/simple/x/"
        );
    }

    #[test]
    fn strip_user_pass() {
        assert_eq!(
            strip_credentials("https://user:pw@host/path"),
            "https://host/path"
        );
    }

    #[test]
    fn strip_user_only() {
        assert_eq!(
            strip_credentials("https://token@host/path"),
            "https://host/path"
        );
    }

    #[test]
    fn strip_preserves_port() {
        assert_eq!(
            strip_credentials("https://u:p@host:8080/path"),
            "https://host:8080/path"
        );
    }

    #[test]
    fn strip_preserves_query_and_fragment() {
        assert_eq!(
            strip_credentials("https://u:p@host/path?q=1#frag"),
            "https://host/path?q=1#frag"
        );
    }

    #[test]
    fn strip_no_credentials_passes_through() {
        assert_eq!(
            strip_credentials("https://pypi.org/simple/x/"),
            "https://pypi.org/simple/x/"
        );
    }

    #[test]
    fn no_scheme_passes_through() {
        // `git@github.com:user/repo.git` is an SSH-style ref. We
        // intentionally pass through — callers handle it.
        let ssh = "git@github.com:user/repo.git";
        assert_eq!(redact_credentials(ssh), ssh);
        assert_eq!(strip_credentials(ssh), ssh);
    }

    #[test]
    fn file_path_passes_through() {
        let p = "/opt/cache/x.whl";
        assert_eq!(redact_credentials(p), p);
        assert_eq!(strip_credentials(p), p);
    }

    #[test]
    fn empty_string_passes_through() {
        assert_eq!(redact_credentials(""), "");
        assert_eq!(strip_credentials(""), "");
    }

    #[test]
    fn http_scheme_handled() {
        assert_eq!(
            strip_credentials("http://u:p@local/idx"),
            "http://local/idx"
        );
    }

    #[test]
    fn authority_only_no_path() {
        assert_eq!(redact_credentials("https://u:p@host"), "https://****@host");
        assert_eq!(strip_credentials("https://u:p@host"), "https://host");
    }

    #[test]
    fn at_sign_in_path_not_touched() {
        // Path-segment `@` is legal in RFC 3986 and must not be
        // mistaken for userinfo terminator.
        let url = "https://host/path/a@b";
        assert_eq!(redact_credentials(url), url);
        assert_eq!(strip_credentials(url), url);
    }

    #[test]
    fn at_sign_in_query_not_touched() {
        let url = "https://host/path?email=a@b.com";
        assert_eq!(redact_credentials(url), url);
        assert_eq!(strip_credentials(url), url);
    }

    #[test]
    fn at_sign_in_userinfo_works_with_last_at_wins() {
        // RFC 3986 lets `@` appear pct-encoded in userinfo; if a
        // producer writes a literal `@` in the user (technically
        // illegal but seen in the wild), the LAST `@` is the
        // terminator.
        let url = "https://name@example.com:secret@host/x";
        assert_eq!(strip_credentials(url), "https://host/x");
        assert_eq!(redact_credentials(url), "https://****@host/x");
    }

    #[test]
    fn percent_encoded_credentials_redacted_wholesale() {
        // We don't decode — pct-encoded `:` and `/` in userinfo stay
        // hidden behind the redaction.
        assert_eq!(
            redact_credentials("https://u%3Aname:p%2Fass@host/x"),
            "https://****@host/x"
        );
        assert_eq!(
            strip_credentials("https://u%3Aname:p%2Fass@host/x"),
            "https://host/x"
        );
    }

    #[test]
    fn idempotent_on_already_redacted() {
        let once = redact_credentials("https://u:p@host/x");
        assert_eq!(redact_credentials(&once), once);
    }

    #[test]
    fn idempotent_on_already_stripped() {
        let once = strip_credentials("https://u:p@host/x");
        assert_eq!(strip_credentials(&once), once);
    }

    #[test]
    fn cache_key_property_strip_equals_clean() {
        // Same URL with and without creds must produce the same
        // stripped form — the property we rely on for cache keys.
        let with_creds = "https://u:p@pypi.org/simple/requests/";
        let no_creds = "https://pypi.org/simple/requests/";
        assert_eq!(strip_credentials(with_creds), no_creds);
        assert_eq!(strip_credentials(no_creds), no_creds);
    }
}
