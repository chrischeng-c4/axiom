use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/copy/custom_copy_hook_propagates.py`.
#[test]
fn test_gen_errors_std_libs_copy_custom_copy_hook_propagates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "errors"
# case = "custom_copy_hook_propagates"
# subject = "copy.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: custom_copy_hook_propagates (errors)."""
import copy

_raised = False
try:
    copy.copy(type('BadCopy', (), {'__copy__': lambda self: (_ for _ in ()).throw(copy.Error('refused'))})())
except copy.Error:
    _raised = True
assert _raised, "custom_copy_hook_propagates: expected copy.Error"
print("custom_copy_hook_propagates OK")
"###);
    assert_output(&out, r###"custom_copy_hook_propagates OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/copy/custom_deepcopy_hook_propagates.py`.
#[test]
fn test_gen_errors_std_libs_copy_custom_deepcopy_hook_propagates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "errors"
# case = "custom_deepcopy_hook_propagates"
# subject = "copy.deepcopy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: custom_deepcopy_hook_propagates (errors)."""
import copy

_raised = False
try:
    copy.deepcopy(type('BadDeepCopy', (), {'__deepcopy__': lambda self, memo: (_ for _ in ()).throw(copy.Error('refused'))})())
except copy.Error:
    _raised = True
assert _raised, "custom_deepcopy_hook_propagates: expected copy.Error"
print("custom_deepcopy_hook_propagates OK")
"###);
    assert_output(&out, r###"custom_deepcopy_hook_propagates OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/copy/getstate_exception_propagates.py`.
#[test]
fn test_gen_errors_std_libs_copy_getstate_exception_propagates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "errors"
# case = "getstate_exception_propagates"
# subject = "copy.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: getstate_exception_propagates (errors)."""
import copy

_raised = False
try:
    copy.copy(type('EvilState', (), {'__getstate__': lambda self: (_ for _ in ()).throw(ValueError('no state'))})())
except ValueError:
    _raised = True
assert _raised, "getstate_exception_propagates: expected ValueError"
print("getstate_exception_propagates OK")
"###);
    assert_output(&out, r###"getstate_exception_propagates OK
"###);
}
