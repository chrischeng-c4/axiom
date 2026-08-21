//! Py3.12 conformance tests for the `textwrap` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_textwrap.py):
//!   dedent (leading-whitespace removal). textwrap.fill and
//!   textwrap.shorten are intentionally excluded — mamba's
//!   implementation currently passes input through unchanged
//!   (no actual wrapping/shortening). Deferred as a separate gap.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_textwrap_dedent_uniform_indent() {
    let out = jit_capture(
        r#"import textwrap
print(textwrap.dedent("    line1\n    line2"))
"#,
    );
    assert_output(&out, "line1\nline2\n");
}

#[test]
fn test_textwrap_dedent_no_indent_passthrough() {
    let out = jit_capture(
        r#"import textwrap
print(textwrap.dedent("line1\nline2"))
"#,
    );
    assert_output(&out, "line1\nline2\n");
}

#[test]
fn test_textwrap_dedent_empty_string() {
    let out = jit_capture(
        r#"import textwrap
print(repr(textwrap.dedent("")))
"#,
    );
    assert_output(&out, "''\n");
}
