// Canonical PEP 503 name normalization (Tick 50 organize pass).
//
// Many modules in this crate previously carried their own copies of
// `pep503_normalize`. They are merged here so there's a single home for
// the rule and all callers stay byte-for-byte consistent.
//
// PEP 503 §"Normalized Names":
//   "lowercased and any runs of the characters `-`, `_` or `.` are
//    replaced with a single `-`."
// Leading and trailing separators are trimmed.

/// PEP 503 normalize: lowercase, collapse `-_. ` runs to a single `-`,
/// trim leading/trailing separators.
pub fn pep503_normalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_sep = false;
    for c in name.chars() {
        let lc = c.to_ascii_lowercase();
        if lc == '-' || lc == '_' || lc == '.' {
            if !last_sep {
                out.push('-');
            }
            last_sep = true;
        } else {
            out.push(lc);
            last_sep = false;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases_input() {
        assert_eq!(pep503_normalize("Requests"), "requests");
    }

    #[test]
    fn collapses_separators_to_single_dash() {
        assert_eq!(pep503_normalize("my_pkg"), "my-pkg");
        assert_eq!(pep503_normalize("my.pkg"), "my-pkg");
        assert_eq!(pep503_normalize("my-pkg"), "my-pkg");
    }

    #[test]
    fn collapses_runs_of_mixed_separators() {
        assert_eq!(pep503_normalize("foo_-_bar"), "foo-bar");
        assert_eq!(pep503_normalize("foo..bar"), "foo-bar");
        assert_eq!(pep503_normalize("foo.-_bar"), "foo-bar");
    }

    #[test]
    fn trims_leading_and_trailing_separators() {
        assert_eq!(pep503_normalize("_foo_"), "foo");
        assert_eq!(pep503_normalize(".-_pkg.-_"), "pkg");
    }

    #[test]
    fn preserves_alphanumerics() {
        assert_eq!(pep503_normalize("pkg123"), "pkg123");
        assert_eq!(pep503_normalize("Numpy2"), "numpy2");
    }

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(pep503_normalize(""), "");
        assert_eq!(pep503_normalize("___"), "");
    }
}
