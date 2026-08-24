use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/inspect_traceback/getframeinfo_none_raises.py`.
#[test]
fn test_gen_errors_std_libs_inspect_traceback_getframeinfo_none_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "errors"
# case = "getframeinfo_none_raises"
# subject = "inspect.getframeinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""inspect.getframeinfo: getframeinfo_none_raises (errors)."""
import inspect

_raised = False
try:
    inspect.getframeinfo(None)
except AttributeError:
    _raised = True
assert _raised, "getframeinfo_none_raises: expected AttributeError"
print("getframeinfo_none_raises OK")
"###);
    assert_output(&out, r###"getframeinfo_none_raises OK
"###);
}
