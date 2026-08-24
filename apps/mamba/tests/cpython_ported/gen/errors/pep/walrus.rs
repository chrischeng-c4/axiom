use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/walrus/bare_walrus_statement_rejected.py`.
#[test]
fn test_gen_errors_pep_walrus_bare_walrus_statement_rejected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "errors"
# case = "bare_walrus_statement_rejected"
# subject = ":="
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: bare_walrus_statement_rejected (errors)."""
pass

_raised = False
try:
    exec('a := 5')
except SyntaxError:
    _raised = True
assert _raised, "bare_walrus_statement_rejected: expected SyntaxError"
print("bare_walrus_statement_rejected OK")
"###);
    assert_output(&out, r###"bare_walrus_statement_rejected OK
"###);
}
