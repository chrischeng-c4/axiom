use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/token/tok_name_unknown_key_raises_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_token_tok_name_unknown_key_raises_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "errors"
# case = "tok_name_unknown_key_raises_keyerror"
# subject = "token.tok_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.tok_name: tok_name_unknown_key_raises_keyerror (errors)."""
import token

_raised = False
try:
    token.tok_name[99999]
except KeyError:
    _raised = True
assert _raised, "tok_name_unknown_key_raises_keyerror: expected KeyError"
print("tok_name_unknown_key_raises_keyerror OK")
"###);
    assert_output(&out, r###"tok_name_unknown_key_raises_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/token/unknown_attribute_raises_attributeerror.py`.
#[test]
fn test_gen_errors_std_libs_token_unknown_attribute_raises_attributeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "errors"
# case = "unknown_attribute_raises_attributeerror"
# subject = "token"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token: unknown_attribute_raises_attributeerror (errors)."""
import token

_raised = False
try:
    token.NO_SUCH_TOKEN_XYZZY
except AttributeError:
    _raised = True
assert _raised, "unknown_attribute_raises_attributeerror: expected AttributeError"
print("unknown_attribute_raises_attributeerror OK")
"###);
    assert_output(&out, r###"unknown_attribute_raises_attributeerror OK
"###);
}
