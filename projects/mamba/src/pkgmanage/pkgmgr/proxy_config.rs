// HTTP/HTTPS proxy env-var parser (Tick 127).
//
// pip, uv, requests, curl, and every HTTP client running inside a
// corporate network share the same env-var contract for proxy
// configuration:
//
//   HTTP_PROXY   (or http_proxy)   — proxy URL for plain-HTTP targets
//   HTTPS_PROXY  (or https_proxy)  — proxy URL for HTTPS targets
//   ALL_PROXY    (or all_proxy)    — fallback for both
//   NO_PROXY     (or no_proxy)     — comma-separated bypass list
//
// Lowercase forms take precedence in libcurl; in Python's urllib /
// requests / uv the uppercase forms typically win on POSIX, but the
// lowercase forms win on Windows (where env vars are case-insensitive).
// We honor the lowercase form first if both are set with different
// values, then fall back to uppercase — matching curl's behavior, which
// is what `pip`'s requests dependency inherits.
//
// NO_PROXY entries are comma-separated. Each entry is one of:
//   * `*`                  — bypass everything
//   * `host` or `.host`    — suffix match against the target hostname
//   * `host:port`          — same, but only for the specified port
//   * IP literals          — exact match (no CIDR; left to the network
//                            stack so we avoid pulling in an IP crate)
//
// This module is a pure parser + matcher. The actual HTTP-client
// integration (telling reqwest/hyper which proxy to use) is in
// `http.rs`.

const PROXY_DETAIL: &str = "<proxy URL>";

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Decoded proxy URL.
///
/// We do not fully parse the URL into structural pieces — that's the
/// HTTP client's job. Instead we keep the verbatim URL and a few
/// derived bits used by the bypass matcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyUrl {
    /// Original URL text, suitable for handing to `reqwest::Proxy::all`.
    pub url: String,
    /// Lowercased scheme (`http`, `https`, `socks5`, `socks5h`, …).
    pub scheme: String,
    /// Lowercased host portion.
    pub host: String,
    /// Explicit port if present in the URL.
    pub port: Option<u16>,
    /// Userinfo half of the URL, if any. Stored verbatim — caller
    /// decides whether to redact it for logs (`url_redact.rs` is the
    /// right home for that).
    pub userinfo: Option<String>,
}

impl ProxyUrl {
    /// Parse a proxy URL of the form `scheme://[user[:pass]@]host[:port]`.
    /// A bare `host:port` (no scheme) is implicitly treated as `http://`,
    /// matching curl's leniency.
    pub fn parse(raw: &str) -> Result<Self, IndexError> {
        let raw_trimmed = raw.trim();
        if raw_trimmed.is_empty() {
            return Err(IndexError::ParseError {
                url: PROXY_DETAIL.into(),
                detail: "empty proxy URL".into(),
            });
        }

        let (canonical, scheme, body) = match raw_trimmed.find("://") {
            Some(idx) => {
                let scheme = raw_trimmed[..idx].to_ascii_lowercase();
                let body = &raw_trimmed[idx + 3..];
                (raw_trimmed.to_string(), scheme, body)
            }
            None => {
                // No scheme — bolt on `http://`.
                let canonical = format!("http://{raw_trimmed}");
                (canonical, "http".to_string(), raw_trimmed)
            }
        };

        if scheme.is_empty() {
            return Err(IndexError::ParseError {
                url: PROXY_DETAIL.into(),
                detail: "proxy URL has empty scheme".into(),
            });
        }

        // Split userinfo (`user:pass@`) from the host:port body.
        let (userinfo, hostport) = match body.find('@') {
            Some(idx) => (Some(body[..idx].to_string()), &body[idx + 1..]),
            None => (None, body),
        };

        // Strip any path/query — proxy URLs don't use them, but curl
        // tolerates a trailing `/`.
        let hostport = hostport.split(['/', '?', '#']).next().unwrap_or(hostport);

        if hostport.is_empty() {
            return Err(IndexError::ParseError {
                url: PROXY_DETAIL.into(),
                detail: "proxy URL has empty host".into(),
            });
        }

        // Split host vs port. The host can be a literal IPv6 in `[...]`.
        let (host_raw, port_str) = if let Some(stripped) = hostport.strip_prefix('[') {
            let close = stripped.find(']').ok_or_else(|| IndexError::ParseError {
                url: PROXY_DETAIL.into(),
                detail: "proxy URL has unterminated IPv6 literal".into(),
            })?;
            let host = &stripped[..close];
            let rest = &stripped[close + 1..];
            let port = rest.strip_prefix(':').map(|p| p.to_string());
            (host.to_string(), port)
        } else if let Some(colon) = hostport.rfind(':') {
            // IPv4 / hostname with optional port.
            let host = &hostport[..colon];
            let port = &hostport[colon + 1..];
            (host.to_string(), Some(port.to_string()))
        } else {
            (hostport.to_string(), None)
        };

        if host_raw.is_empty() {
            return Err(IndexError::ParseError {
                url: PROXY_DETAIL.into(),
                detail: "proxy URL has empty host".into(),
            });
        }

        let port = match port_str {
            Some(p) if p.is_empty() => None,
            Some(p) => Some(p.parse::<u16>().map_err(|_| IndexError::ParseError {
                url: PROXY_DETAIL.into(),
                detail: format!("proxy URL has non-numeric port {p:?}"),
            })?),
            None => None,
        };

        Ok(ProxyUrl {
            url: canonical,
            scheme,
            host: host_raw.to_ascii_lowercase(),
            port,
            userinfo,
        })
    }
}

/// Parsed NO_PROXY bypass list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoProxyList {
    /// `*` was seen — bypass every host.
    pub all_wildcard: bool,
    /// (host_suffix_lower, optional_port) entries.
    pub entries: Vec<NoProxyEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoProxyEntry {
    /// Lowercased host suffix, without a leading dot.
    pub host_suffix: String,
    /// Explicit port restriction, if the entry was `host:port`.
    pub port: Option<u16>,
}

impl NoProxyList {
    /// Parse the body of a `NO_PROXY` env var. Comma-separated; entries
    /// trimmed; empty entries dropped.
    pub fn parse(raw: &str) -> Self {
        let mut all_wildcard = false;
        let mut entries = Vec::new();
        for part in raw.split(',') {
            let entry = part.trim();
            if entry.is_empty() {
                continue;
            }
            if entry == "*" {
                all_wildcard = true;
                continue;
            }
            // Strip leading dot — `example.com` and `.example.com`
            // mean the same thing (suffix match).
            let entry = entry.strip_prefix('.').unwrap_or(entry);
            let (host, port) = match entry.rsplit_once(':') {
                Some((h, p)) if p.parse::<u16>().is_ok() => {
                    (h.to_string(), Some(p.parse::<u16>().unwrap()))
                }
                _ => (entry.to_string(), None),
            };
            entries.push(NoProxyEntry {
                host_suffix: host.to_ascii_lowercase(),
                port,
            });
        }
        NoProxyList {
            all_wildcard,
            entries,
        }
    }

    /// True when `host`/`port` matches any bypass entry.
    pub fn matches(&self, host: &str, port: Option<u16>) -> bool {
        if self.all_wildcard {
            return true;
        }
        let host = host.to_ascii_lowercase();
        for entry in &self.entries {
            if !host_matches_suffix(&host, &entry.host_suffix) {
                continue;
            }
            match (entry.port, port) {
                (None, _) => return true,
                (Some(entry_port), Some(target_port)) if entry_port == target_port => return true,
                _ => continue,
            }
        }
        false
    }
}

/// Aggregate proxy configuration assembled from env vars.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyConfig {
    pub http_proxy: Option<ProxyUrl>,
    pub https_proxy: Option<ProxyUrl>,
    pub all_proxy: Option<ProxyUrl>,
    pub no_proxy: NoProxyList,
}

impl ProxyConfig {
    /// Build a config from an env-var lookup closure. Caller passes the
    /// closure rather than this module reading `std::env` directly,
    /// which keeps the module unit-testable without touching process
    /// env. The closure should be case-sensitive — we probe each
    /// recognized name explicitly.
    pub fn from_env<F>(get: F) -> Result<Self, IndexError>
    where
        F: Fn(&str) -> Option<String>,
    {
        // Lowercase first (curl precedence), then uppercase fallback.
        let pick =
            |lower: &str, upper: &str| -> Option<String> { get(lower).or_else(|| get(upper)) };

        let http_proxy = match pick("http_proxy", "HTTP_PROXY") {
            Some(s) if !s.trim().is_empty() => Some(ProxyUrl::parse(&s)?),
            _ => None,
        };
        let https_proxy = match pick("https_proxy", "HTTPS_PROXY") {
            Some(s) if !s.trim().is_empty() => Some(ProxyUrl::parse(&s)?),
            _ => None,
        };
        let all_proxy = match pick("all_proxy", "ALL_PROXY") {
            Some(s) if !s.trim().is_empty() => Some(ProxyUrl::parse(&s)?),
            _ => None,
        };
        let no_proxy = pick("no_proxy", "NO_PROXY")
            .map(|s| NoProxyList::parse(&s))
            .unwrap_or_default();

        Ok(ProxyConfig {
            http_proxy,
            https_proxy,
            all_proxy,
            no_proxy,
        })
    }

    /// Choose the proxy that applies to `target_url`. Respects
    /// `NO_PROXY` first, then falls back to scheme-specific
    /// (`HTTPS_PROXY` for `https://`, `HTTP_PROXY` for `http://`),
    /// then `ALL_PROXY`.
    pub fn pick_proxy_for(&self, target_url: &str) -> Option<&ProxyUrl> {
        let (scheme, host, port) = parse_target_url(target_url)?;
        if self.no_proxy.matches(&host, port) {
            return None;
        }
        match scheme.as_str() {
            "https" => self.https_proxy.as_ref().or(self.all_proxy.as_ref()),
            "http" => self.http_proxy.as_ref().or(self.all_proxy.as_ref()),
            _ => self.all_proxy.as_ref(),
        }
    }
}

/// `host` matches `suffix` either by exact match or as a dot-bounded
/// suffix (so `example.com` matches `foo.example.com` but not
/// `notexample.com`).
fn host_matches_suffix(host: &str, suffix: &str) -> bool {
    if host == suffix {
        return true;
    }
    if host.len() <= suffix.len() {
        return false;
    }
    let (head, tail) = host.split_at(host.len() - suffix.len());
    tail == suffix && head.ends_with('.')
}

/// Best-effort URL split for the matcher only — returns (scheme,
/// host, port). Not a full RFC 3986 parser; if it can't find the
/// `scheme://host` pieces it returns None and the caller treats the
/// URL as opaque (no bypass match).
fn parse_target_url(url: &str) -> Option<(String, String, Option<u16>)> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() {
        return None;
    }
    // Strip userinfo.
    let authority = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let (host, port) = if let Some(stripped) = authority.strip_prefix('[') {
        let close = stripped.find(']')?;
        let host = &stripped[..close];
        let rest = &stripped[close + 1..];
        let port = rest.strip_prefix(':').and_then(|p| p.parse::<u16>().ok());
        (host.to_string(), port)
    } else if let Some(colon) = authority.rfind(':') {
        let port = authority[colon + 1..].parse::<u16>().ok();
        let host = &authority[..colon];
        (host.to_string(), port)
    } else {
        (authority.to_string(), None)
    };
    Some((scheme, host.to_ascii_lowercase(), port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn lookup(map: HashMap<String, String>) -> impl Fn(&str) -> Option<String> {
        move |k| map.get(k).cloned()
    }

    #[test]
    fn parses_full_proxy_url() {
        let p = ProxyUrl::parse("http://user:pass@proxy.corp:8080").unwrap();
        assert_eq!(p.scheme, "http");
        assert_eq!(p.host, "proxy.corp");
        assert_eq!(p.port, Some(8080));
        assert_eq!(p.userinfo.as_deref(), Some("user:pass"));
        assert_eq!(p.url, "http://user:pass@proxy.corp:8080");
    }

    #[test]
    fn parses_bare_host_port_as_http() {
        let p = ProxyUrl::parse("proxy.corp:8080").unwrap();
        assert_eq!(p.scheme, "http");
        assert_eq!(p.host, "proxy.corp");
        assert_eq!(p.port, Some(8080));
    }

    #[test]
    fn parses_ipv6_proxy_url() {
        let p = ProxyUrl::parse("http://[::1]:8888").unwrap();
        assert_eq!(p.scheme, "http");
        assert_eq!(p.host, "::1");
        assert_eq!(p.port, Some(8888));
    }

    #[test]
    fn lowercases_scheme_and_host() {
        let p = ProxyUrl::parse("HTTP://PROXY.Corp:80").unwrap();
        assert_eq!(p.scheme, "http");
        assert_eq!(p.host, "proxy.corp");
    }

    #[test]
    fn rejects_empty_url() {
        assert!(ProxyUrl::parse("").is_err());
        assert!(ProxyUrl::parse("   ").is_err());
    }

    #[test]
    fn rejects_non_numeric_port() {
        assert!(ProxyUrl::parse("http://proxy.corp:notaport").is_err());
    }

    #[test]
    fn rejects_unterminated_ipv6() {
        assert!(ProxyUrl::parse("http://[::1").is_err());
    }

    #[test]
    fn no_proxy_wildcard_matches_everything() {
        let n = NoProxyList::parse("*");
        assert!(n.all_wildcard);
        assert!(n.matches("example.com", None));
        assert!(n.matches("internal.corp", Some(443)));
    }

    #[test]
    fn no_proxy_suffix_match_with_dot_boundary() {
        let n = NoProxyList::parse("example.com");
        assert!(n.matches("example.com", None));
        assert!(n.matches("api.example.com", None));
        assert!(!n.matches("notexample.com", None));
        assert!(!n.matches("example.com.evil.io", None));
    }

    #[test]
    fn no_proxy_leading_dot_is_equivalent_to_no_dot() {
        let n1 = NoProxyList::parse(".example.com");
        let n2 = NoProxyList::parse("example.com");
        assert_eq!(n1, n2);
    }

    #[test]
    fn no_proxy_port_specific_entry_only_matches_that_port() {
        let n = NoProxyList::parse("example.com:8080");
        assert!(n.matches("example.com", Some(8080)));
        assert!(!n.matches("example.com", Some(443)));
        assert!(!n.matches("example.com", None));
    }

    #[test]
    fn no_proxy_drops_empty_entries() {
        let n = NoProxyList::parse(", , , ,example.com,, ");
        assert_eq!(n.entries.len(), 1);
        assert_eq!(n.entries[0].host_suffix, "example.com");
    }

    #[test]
    fn no_proxy_lowercases_hosts() {
        let n = NoProxyList::parse("EXAMPLE.COM");
        assert!(n.matches("example.com", None));
        assert!(n.matches("EXAMPLE.COM", None));
    }

    #[test]
    fn from_env_lowercase_takes_precedence() {
        let map = env(&[
            ("http_proxy", "http://lower.corp:8080"),
            ("HTTP_PROXY", "http://upper.corp:9090"),
        ]);
        let cfg = ProxyConfig::from_env(lookup(map)).unwrap();
        assert_eq!(cfg.http_proxy.unwrap().host, "lower.corp");
    }

    #[test]
    fn from_env_falls_back_to_uppercase() {
        let map = env(&[("HTTPS_PROXY", "http://upper.corp:9090")]);
        let cfg = ProxyConfig::from_env(lookup(map)).unwrap();
        assert_eq!(cfg.https_proxy.unwrap().host, "upper.corp");
    }

    #[test]
    fn from_env_skips_empty_strings() {
        let map = env(&[("HTTP_PROXY", ""), ("https_proxy", "   ")]);
        let cfg = ProxyConfig::from_env(lookup(map)).unwrap();
        assert!(cfg.http_proxy.is_none());
        assert!(cfg.https_proxy.is_none());
    }

    #[test]
    fn pick_proxy_for_uses_scheme_specific() {
        let cfg = ProxyConfig::from_env(lookup(env(&[
            ("HTTP_PROXY", "http://hp.corp:80"),
            ("HTTPS_PROXY", "http://hps.corp:443"),
        ])))
        .unwrap();
        assert_eq!(
            cfg.pick_proxy_for("http://example.com/foo").unwrap().host,
            "hp.corp"
        );
        assert_eq!(
            cfg.pick_proxy_for("https://example.com/foo").unwrap().host,
            "hps.corp"
        );
    }

    #[test]
    fn pick_proxy_for_falls_back_to_all_proxy() {
        let cfg = ProxyConfig::from_env(lookup(env(&[("ALL_PROXY", "socks5://socks.corp:1080")])))
            .unwrap();
        let p = cfg.pick_proxy_for("https://example.com").unwrap();
        assert_eq!(p.scheme, "socks5");
    }

    #[test]
    fn pick_proxy_for_respects_no_proxy_bypass() {
        let cfg = ProxyConfig::from_env(lookup(env(&[
            ("HTTPS_PROXY", "http://hps.corp:443"),
            ("NO_PROXY", "example.com"),
        ])))
        .unwrap();
        assert!(cfg.pick_proxy_for("https://api.example.com").is_none());
        assert!(cfg.pick_proxy_for("https://other.io").is_some());
    }

    #[test]
    fn pick_proxy_for_returns_none_when_no_proxies_set() {
        let cfg = ProxyConfig::default();
        assert!(cfg.pick_proxy_for("https://example.com").is_none());
    }

    #[test]
    fn pick_proxy_for_returns_none_for_malformed_target() {
        let cfg =
            ProxyConfig::from_env(lookup(env(&[("ALL_PROXY", "http://all.corp:80")]))).unwrap();
        assert!(cfg.pick_proxy_for("not-a-url").is_none());
    }

    #[test]
    fn realistic_corporate_env_picks_correctly() {
        // Common corporate setup: HTTP/HTTPS proxies + a NO_PROXY
        // covering internal/loopback.
        let cfg = ProxyConfig::from_env(lookup(env(&[
            ("HTTP_PROXY", "http://proxy.corp:8080"),
            ("HTTPS_PROXY", "http://proxy.corp:8080"),
            ("NO_PROXY", "localhost,127.0.0.1,.corp,internal.io"),
        ])))
        .unwrap();
        assert!(cfg.pick_proxy_for("https://pypi.org/simple/").is_some());
        assert!(cfg.pick_proxy_for("http://localhost:5000").is_none());
        assert!(cfg.pick_proxy_for("https://artifacts.corp/repo").is_none());
        assert!(cfg.pick_proxy_for("https://api.internal.io/v1").is_none());
    }

    #[test]
    fn proxy_url_with_path_strips_path() {
        let p = ProxyUrl::parse("http://proxy.corp:8080/some/path").unwrap();
        assert_eq!(p.host, "proxy.corp");
        assert_eq!(p.port, Some(8080));
    }

    #[test]
    fn invalid_proxy_value_surfaces_as_error_from_from_env() {
        let map = env(&[("HTTP_PROXY", "http://[invalid-ipv6")]);
        assert!(ProxyConfig::from_env(lookup(map)).is_err());
    }
}
