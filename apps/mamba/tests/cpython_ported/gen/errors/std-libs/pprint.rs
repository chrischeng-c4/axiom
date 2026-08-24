use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/pprint/negative_depth_raises.py`.
#[test]
fn test_gen_errors_std_libs_pprint_negative_depth_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "negative_depth_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.PrettyPrinter: negative_depth_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(depth=-1)
except ValueError:
    _raised = True
assert _raised, "negative_depth_raises: expected ValueError"
print("negative_depth_raises OK")
"###);
    assert_output(&out, r###"negative_depth_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pprint/negative_indent_raises.py`.
#[test]
fn test_gen_errors_std_libs_pprint_negative_indent_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "negative_indent_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: negative_indent_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(indent=-1)
except ValueError:
    _raised = True
assert _raised, "negative_indent_raises: expected ValueError"
print("negative_indent_raises OK")
"###);
    assert_output(&out, r###"negative_indent_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pprint/negative_width_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_pprint_negative_width_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "negative_width_no_raise"
# subject = "pprint.PrettyPrinter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.PrettyPrinter: PrettyPrinter(width=-1) does NOT raise under CPython 3.12 (only depth/indent and zero width/depth are validated); constructing it succeeds"""
import pprint

# Unlike depth/indent and a zero width/depth, a negative width is NOT
# validated by the constructor, so this must succeed without raising.
pp = pprint.PrettyPrinter(width=-1)
assert isinstance(pp, pprint.PrettyPrinter)
print("negative_width_no_raise OK")
"###);
    assert_output(&out, r###"negative_width_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pprint/unorderable_keys_sort_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_pprint_unorderable_keys_sort_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "unorderable_keys_sort_no_raise"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat({1:'a','two':'b'}, sort_dicts=True) does NOT raise: pprint catches the unorderable-key TypeError internally and falls back to insertion order"""
import pprint

# int vs str keys are unorderable, but pprint catches the TypeError from the
# attempted sort and falls back to insertion order instead of propagating it.
mixed = {1: "a", "two": "b"}
out = pprint.pformat(mixed, sort_dicts=True)
assert out == "{1: 'a', 'two': 'b'}", out
print("unorderable_keys_sort_no_raise OK")
"###);
    assert_output(&out, r###"unorderable_keys_sort_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pprint/zero_depth_raises.py`.
#[test]
fn test_gen_errors_std_libs_pprint_zero_depth_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "zero_depth_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: zero_depth_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(depth=0)
except ValueError:
    _raised = True
assert _raised, "zero_depth_raises: expected ValueError"
print("zero_depth_raises OK")
"###);
    assert_output(&out, r###"zero_depth_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/pprint/zero_width_raises.py`.
#[test]
fn test_gen_errors_std_libs_pprint_zero_width_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "zero_width_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: zero_width_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(width=0)
except ValueError:
    _raised = True
assert _raised, "zero_width_raises: expected ValueError"
print("zero_width_raises OK")
"###);
    assert_output(&out, r###"zero_width_raises OK
"###);
}
