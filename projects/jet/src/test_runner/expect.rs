// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! `expect()` matcher catalogue — Rust-side reference for diagnostics.
//!
//! The actual matchers run inside the Node.js worker (see
//! `runtime/test/index.js`); this module documents the contract and provides
//! a small helper for pretty-printing matcher diffs in the Rust reporter.
//!
//! Phase 3 adds three DOM-integrated matchers:
//!
//! - `toHaveText`     — asserts `locator.text_content()` matches expected value
//! - `toBeVisible`    — asserts `locator.is_visible()` returns true
//! - `toMatchSnapshot`— asserts `page.screenshot()` matches a stored golden PNG

/// Matchers shipped in the runner (8 total after Phase 3).
///
/// The first five are pure-JS matchers that run without RPC. The last three
/// issue `expect.*` RPC calls to the Rust host over the wire.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R1
// @spec ...#R2
// @spec ...#R3
pub const CORE_MATCHERS: &[&str] = &[
    // Pure-JS matchers (Phase 1-2).
    "toBe",
    "toEqual",
    "toBeTruthy",
    "toContain",
    "toMatch",
    // DOM-integrated matchers (Phase 3) — require browser RPC.
    "toHaveText",
    "toBeVisible",
    "toMatchSnapshot",
    // Text/object snapshot — #2713.
    "toMatchTextSnapshot",
];

/// Pretty-print a unified-ish diff for matcher failures. Accepts the raw diff
/// string that the worker produces and returns it with consistent indent.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub fn format_diff(diff: &str) -> String {
    diff.lines()
        .map(|l| format!("    {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_matchers_listed() {
        assert!(CORE_MATCHERS.contains(&"toBe"));
        // Phase 3 adds toHaveText, toBeVisible, toMatchSnapshot — total 8.
        // #2713 adds toMatchTextSnapshot — total 9.
        assert_eq!(CORE_MATCHERS.len(), 9);
        assert!(CORE_MATCHERS.contains(&"toHaveText"));
        assert!(CORE_MATCHERS.contains(&"toBeVisible"));
        assert!(CORE_MATCHERS.contains(&"toMatchSnapshot"));
        assert!(CORE_MATCHERS.contains(&"toMatchTextSnapshot"));
    }

    #[test]
    fn format_diff_indents() {
        let s = format_diff("-1\n+2");
        assert_eq!(s, "    -1\n    +2");
    }

    #[test]
    fn format_diff_empty() {
        // An empty diff has no lines to indent → empty string.
        assert_eq!(format_diff(""), "");
    }
}
// CODEGEN-END
