use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/difflib/get_close_matches_high_cutoff_raises.py`.
#[test]
fn test_gen_errors_std_libs_difflib_get_close_matches_high_cutoff_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "get_close_matches_high_cutoff_raises"
# subject = "difflib.get_close_matches"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches_high_cutoff_raises (errors)."""
import difflib

_raised = False
try:
    difflib.get_close_matches("a", ["a"], cutoff=1.5)
except ValueError:
    _raised = True
assert _raised, "get_close_matches_high_cutoff_raises: expected ValueError"
print("get_close_matches_high_cutoff_raises OK")
"###);
    assert_output(&out, r###"get_close_matches_high_cutoff_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/difflib/get_close_matches_negative_cutoff_raises.py`.
#[test]
fn test_gen_errors_std_libs_difflib_get_close_matches_negative_cutoff_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "get_close_matches_negative_cutoff_raises"
# subject = "difflib.get_close_matches"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.get_close_matches: get_close_matches_negative_cutoff_raises (errors)."""
import difflib

_raised = False
try:
    difflib.get_close_matches("a", ["a"], cutoff=-0.1)
except ValueError:
    _raised = True
assert _raised, "get_close_matches_negative_cutoff_raises: expected ValueError"
print("get_close_matches_negative_cutoff_raises OK")
"###);
    assert_output(&out, r###"get_close_matches_negative_cutoff_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/difflib/unified_diff_negative_n_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_difflib_unified_diff_negative_n_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "errors"
# case = "unified_diff_negative_n_no_raise"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff(n=-1) does NOT raise; it just yields a clamped (empty-context) diff"""
import difflib

res = list(difflib.unified_diff(["a"], ["b"], n=-1))
assert isinstance(res, list), f"result type = {type(res)!r}"
print("neg_n: lines=", len(res))
print("unified_diff_negative_n_no_raise OK")
"###);
    assert_output(&out, r###"neg_n: lines= 5
unified_diff_negative_n_no_raise OK
"###);
}
