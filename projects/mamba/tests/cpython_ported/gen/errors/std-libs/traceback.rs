use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/traceback/extract_tb_non_traceback_raises.py`.
#[test]
fn test_gen_errors_std_libs_traceback_extract_tb_non_traceback_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "errors"
# case = "extract_tb_non_traceback_raises"
# subject = "traceback.extract_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb_non_traceback_raises (errors)."""
import traceback

_raised = False
try:
    traceback.extract_tb(123)
except AttributeError:
    _raised = True
assert _raised, "extract_tb_non_traceback_raises: expected AttributeError"
print("extract_tb_non_traceback_raises OK")
"###);
    assert_output(&out, r###"extract_tb_non_traceback_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/traceback/format_exception_mixed_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_traceback_format_exception_mixed_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "errors"
# case = "format_exception_mixed_args_raises"
# subject = "traceback.format_exception"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: format_exception_mixed_args_raises (errors)."""
import traceback

_raised = False
try:
    traceback.format_exception(Exception, Exception('x'))
except ValueError:
    _raised = True
assert _raised, "format_exception_mixed_args_raises: expected ValueError"
print("format_exception_mixed_args_raises OK")
"###);
    assert_output(&out, r###"format_exception_mixed_args_raises OK
"###);
}
