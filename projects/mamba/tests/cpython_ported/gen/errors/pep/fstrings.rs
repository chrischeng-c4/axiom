use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/fstrings/bad_format_spec_raises.py`.
#[test]
fn test_gen_errors_pep_fstrings_bad_format_spec_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "errors"
# case = "bad_format_spec_raises"
# subject = "fstring.format_spec"
# kind = "mechanical"
# xfail = "mamba eval() defers parsing past the strict-typing gate; an invalid format code returns None instead of raising ValueError (project_mamba_pep_silent_divergences_2026_05_27, project_mamba_eval_silent_none_cross_type)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.format_spec: bad_format_spec_raises (errors)."""
# an invalid presentation type in the format spec raises ValueError

_raised = False
try:
    eval('f"{1:Q}"')
except ValueError:
    _raised = True
assert _raised, "bad_format_spec_raises: expected ValueError"
print("bad_format_spec_raises OK")
"###);
    assert_output(&out, r###"bad_format_spec_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/fstrings/empty_expression_raises.py`.
#[test]
fn test_gen_errors_pep_fstrings_empty_expression_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "errors"
# case = "empty_expression_raises"
# subject = "fstring.syntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.syntax: empty_expression_raises (errors)."""
# an empty replacement field {} is a SyntaxError

_raised = False
try:
    eval('f"{}"')
except SyntaxError:
    _raised = True
assert _raised, "empty_expression_raises: expected SyntaxError"
print("empty_expression_raises OK")
"###);
    assert_output(&out, r###"empty_expression_raises OK
"###);
}
