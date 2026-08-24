use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/warnings/bad_regex_raises.py`.
#[test]
fn test_gen_errors_std_libs_warnings_bad_regex_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "bad_regex_raises"
# subject = "warnings.filterwarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: bad_regex_raises (errors)."""
import re
import warnings

_raised = False
try:
    warnings.filterwarnings("ignore", message="*(")
except re.error:
    _raised = True
assert _raised, "bad_regex_raises: expected re.error"
print("bad_regex_raises OK")
"###);
    assert_output(&out, r###"bad_regex_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/warnings/error_filter_turns_warn_into_raise.py`.
#[test]
fn test_gen_errors_std_libs_warnings_error_filter_turns_warn_into_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "error_filter_turns_warn_into_raise"
# subject = "warnings.simplefilter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: under simplefilter("error") inside a catch_warnings block, warnings.warn raises the warning category as an exception (UserWarning)"""
import warnings

with warnings.catch_warnings():
    warnings.simplefilter("error")
    _raised = False
    try:
        warnings.warn("turned_into_error", UserWarning)
    except UserWarning:
        _raised = True
    assert _raised, "error filter turns warn into a raise"

print("error_filter_turns_warn_into_raise OK")
"###);
    assert_output(&out, r###"error_filter_turns_warn_into_raise OK
"###);
}
