use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/bisect/insort_tuple_raises_attributeerror.py`.
#[test]
fn test_gen_errors_std_libs_bisect_insort_tuple_raises_attributeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "insort_tuple_raises_attributeerror"
# subject = "bisect.insort"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.insort: insort_tuple_raises_attributeerror (errors)."""
import bisect

_raised = False
try:
    bisect.insort((1, 2, 3), 4)
except AttributeError:
    _raised = True
assert _raised, "insort_tuple_raises_attributeerror: expected AttributeError"
print("insort_tuple_raises_attributeerror OK")
"###);
    assert_output(&out, r###"insort_tuple_raises_attributeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bisect/mixed_types_raise_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_bisect_mixed_types_raise_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "mixed_types_raise_typeerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: mixed_types_raise_typeerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left([1, 2, 3], "x")
except TypeError:
    _raised = True
assert _raised, "mixed_types_raise_typeerror: expected TypeError"
print("mixed_types_raise_typeerror OK")
"###);
    assert_output(&out, r###"mixed_types_raise_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bisect/negative_lo_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_bisect_negative_lo_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "negative_lo_raises_valueerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: negative_lo_raises_valueerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left([1, 2, 3], 2, lo=-1)
except ValueError:
    _raised = True
assert _raised, "negative_lo_raises_valueerror: expected ValueError"
print("negative_lo_raises_valueerror OK")
"###);
    assert_output(&out, r###"negative_lo_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/bisect/non_sequence_first_arg_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_bisect_non_sequence_first_arg_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "errors"
# case = "non_sequence_first_arg_raises_typeerror"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: non_sequence_first_arg_raises_typeerror (errors)."""
import bisect

_raised = False
try:
    bisect.bisect_left(10, 10)
except TypeError:
    _raised = True
assert _raised, "non_sequence_first_arg_raises_typeerror: expected TypeError"
print("non_sequence_first_arg_raises_typeerror OK")
"###);
    assert_output(&out, r###"non_sequence_first_arg_raises_typeerror OK
"###);
}
