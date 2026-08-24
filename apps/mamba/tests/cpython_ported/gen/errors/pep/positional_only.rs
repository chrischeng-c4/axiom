use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/positional_only/kwonly_as_positional_rejected.py`.
#[test]
fn test_gen_errors_pep_positional_only_kwonly_as_positional_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "errors"
# case = "kwonly_as_positional_rejected"
# subject = "*"
# kind = "mechanical"
# xfail = "mamba does not enforce keyword-only parameters: `_k(3, 4)` is accepted silently instead of raising TypeError (project_mamba_function_machinery_silent_divergences #3)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""*: kwonly_as_positional_rejected (errors)."""
exec("def _k(*, n, m):\n    return n * m", globals())

_raised = False
try:
    _k(3, 4)
except TypeError:
    _raised = True
assert _raised, "kwonly_as_positional_rejected: expected TypeError"
print("kwonly_as_positional_rejected OK")
"###);
    assert_output(&out, r###"kwonly_as_positional_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/positional_only/posonly_as_keyword_rejected.py`.
#[test]
fn test_gen_errors_pep_positional_only_posonly_as_keyword_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "errors"
# case = "posonly_as_keyword_rejected"
# subject = "/"
# kind = "mechanical"
# xfail = "mamba does not enforce positional-only parameters: `_p(a=1, b=2)` is accepted silently instead of raising TypeError (project_mamba_function_machinery_silent_divergences #4)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: posonly_as_keyword_rejected (errors)."""
exec("def _p(a, b, /):\n    return a + b", globals())

_raised = False
try:
    _p(a=1, b=2)
except TypeError:
    _raised = True
assert _raised, "posonly_as_keyword_rejected: expected TypeError"
print("posonly_as_keyword_rejected OK")
"###);
    assert_output(&out, r###"posonly_as_keyword_rejected OK
"###);
}

/// Ported from `tests/cpython/errors/pep/positional_only/solo_slash_rejected.py`.
#[test]
fn test_gen_errors_pep_positional_only_solo_slash_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "errors"
# case = "solo_slash_rejected"
# subject = "/"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: solo_slash_rejected (errors)."""
pass

_raised = False
try:
    compile('def h(/): pass', '<t>', 'exec')
except SyntaxError:
    _raised = True
assert _raised, "solo_slash_rejected: expected SyntaxError"
print("solo_slash_rejected OK")
"###);
    assert_output(&out, r###"solo_slash_rejected OK
"###);
}
