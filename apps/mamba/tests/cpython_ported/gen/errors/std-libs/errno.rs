use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/errno/errorcode_unknown_key_raises.py`.
#[test]
fn test_gen_errors_std_libs_errno_errorcode_unknown_key_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "errors"
# case = "errorcode_unknown_key_raises"
# subject = "errno.errorcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno.errorcode: errorcode_unknown_key_raises (errors)."""
import errno

_raised = False
try:
    errno.errorcode[99999]
except KeyError:
    _raised = True
assert _raised, "errorcode_unknown_key_raises: expected KeyError"
print("errorcode_unknown_key_raises OK")
"###);
    assert_output(&out, r###"errorcode_unknown_key_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/errno/missing_attribute_raises.py`.
#[test]
fn test_gen_errors_std_libs_errno_missing_attribute_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "errors"
# case = "missing_attribute_raises"
# subject = "errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""errno: missing_attribute_raises (errors)."""
import errno

_raised = False
try:
    errno.NO_SUCH_ERRNO
except AttributeError:
    _raised = True
assert _raised, "missing_attribute_raises: expected AttributeError"
print("missing_attribute_raises OK")
"###);
    assert_output(&out, r###"missing_attribute_raises OK
"###);
}
