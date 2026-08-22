use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/contextvars/get_with_default_arg_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_contextvars_get_with_default_arg_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "get_with_default_arg_no_raise"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: get(default) on an unset ContextVar returns the supplied default instead of raising LookupError"""
import contextvars

cv = contextvars.ContextVar("no_default")
# Unset and no constructor default, but get(fallback) supplies one at call time.
assert cv.get(99) == 99, "get(default) returns the supplied fallback when unset"
print("get_with_default_arg_no_raise OK")
"###);
    assert_output(&out, r###"get_with_default_arg_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextvars/reset_wrong_var_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_contextvars_reset_wrong_var_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "reset_wrong_var_raises_valueerror"
# subject = "contextvars.ContextVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: reset_wrong_var_raises_valueerror (errors)."""
import contextvars

_raised = False
try:
    contextvars.ContextVar('e_a').reset(contextvars.ContextVar('e_b').set(1))
except ValueError:
    _raised = True
assert _raised, "reset_wrong_var_raises_valueerror: expected ValueError"
print("reset_wrong_var_raises_valueerror OK")
"###);
    assert_output(&out, r###"reset_wrong_var_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextvars/reuse_token_raises_runtimeerror.py`.
#[test]
fn test_gen_errors_std_libs_contextvars_reuse_token_raises_runtimeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "reuse_token_raises_runtimeerror"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: resetting with a Token a second time (the Token was already consumed) raises RuntimeError"""
import contextvars

cv = contextvars.ContextVar("reuse")
cv.set(1)
tok = cv.set(2)
cv.reset(tok)  # first reset consumes the token
_raised = False
try:
    cv.reset(tok)  # second reset of the same, now-used, token
except RuntimeError:
    _raised = True
assert _raised, "reusing a consumed Token must raise RuntimeError"
print("reuse_token_raises_runtimeerror OK")
"###);
    assert_output(&out, r###"reuse_token_raises_runtimeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextvars/set_then_get_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_contextvars_set_then_get_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "set_then_get_no_raise"
# subject = "contextvars.ContextVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: set() then get() on a no-default ContextVar returns the set value without raising"""
import contextvars

cv = contextvars.ContextVar("no_default")
cv.set("hello")
# No default, but a value is now set, so get() returns it (no LookupError).
assert cv.get() == "hello", "get() after set() returns the set value"
print("set_then_get_no_raise OK")
"###);
    assert_output(&out, r###"set_then_get_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/contextvars/unset_get_raises_lookuperror.py`.
#[test]
fn test_gen_errors_std_libs_contextvars_unset_get_raises_lookuperror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "unset_get_raises_lookuperror"
# subject = "contextvars.ContextVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: unset_get_raises_lookuperror (errors)."""
import contextvars

_raised = False
try:
    contextvars.ContextVar('e_unset').get()
except LookupError:
    _raised = True
assert _raised, "unset_get_raises_lookuperror: expected LookupError"
print("unset_get_raises_lookuperror OK")
"###);
    assert_output(&out, r###"unset_get_raises_lookuperror OK
"###);
}
