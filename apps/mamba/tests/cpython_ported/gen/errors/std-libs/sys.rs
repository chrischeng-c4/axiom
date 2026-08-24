use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/sys/displayhook_no_arg_raises.py`.
#[test]
fn test_gen_errors_std_libs_sys_displayhook_no_arg_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "displayhook_no_arg_raises"
# subject = "sys.__displayhook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__displayhook__: displayhook_no_arg_raises (errors)."""
import sys

_raised = False
try:
    sys.__displayhook__()
except TypeError:
    _raised = True
assert _raised, "displayhook_no_arg_raises: expected TypeError"
print("displayhook_no_arg_raises OK")
"###);
    assert_output(&out, r###"displayhook_no_arg_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sys/excepthook_no_arg_raises.py`.
#[test]
fn test_gen_errors_std_libs_sys_excepthook_no_arg_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "excepthook_no_arg_raises"
# subject = "sys.__excepthook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__excepthook__: excepthook_no_arg_raises (errors)."""
import sys

_raised = False
try:
    sys.__excepthook__()
except TypeError:
    _raised = True
assert _raised, "excepthook_no_arg_raises: expected TypeError"
print("excepthook_no_arg_raises OK")
"###);
    assert_output(&out, r###"excepthook_no_arg_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sys/exit_raises_systemexit.py`.
#[test]
fn test_gen_errors_std_libs_sys_exit_raises_systemexit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "exit_raises_systemexit"
# subject = "sys.exit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exit: exit_raises_systemexit (errors)."""
import sys

_raised = False
try:
    sys.exit(42)
except SystemExit:
    _raised = True
assert _raised, "exit_raises_systemexit: expected SystemExit"
print("exit_raises_systemexit OK")
"###);
    assert_output(&out, r###"exit_raises_systemexit OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sys/missing_attr_raises.py`.
#[test]
fn test_gen_errors_std_libs_sys_missing_attr_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "missing_attr_raises"
# subject = "sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys: missing_attr_raises (errors)."""
import sys

_raised = False
try:
    sys.no_such_attr_xyzzy
except AttributeError:
    _raised = True
assert _raised, "missing_attr_raises: expected AttributeError"
print("missing_attr_raises OK")
"###);
    assert_output(&out, r###"missing_attr_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sys/setrecursionlimit_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_sys_setrecursionlimit_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "setrecursionlimit_negative_raises"
# subject = "sys.setrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit_negative_raises (errors)."""
import sys

_raised = False
try:
    sys.setrecursionlimit(-5)
except ValueError:
    _raised = True
assert _raised, "setrecursionlimit_negative_raises: expected ValueError"
print("setrecursionlimit_negative_raises OK")
"###);
    assert_output(&out, r###"setrecursionlimit_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sys/setrecursionlimit_zero_raises.py`.
#[test]
fn test_gen_errors_std_libs_sys_setrecursionlimit_zero_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "setrecursionlimit_zero_raises"
# subject = "sys.setrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit_zero_raises (errors)."""
import sys

_raised = False
try:
    sys.setrecursionlimit(0)
except ValueError:
    _raised = True
assert _raised, "setrecursionlimit_zero_raises: expected ValueError"
print("setrecursionlimit_zero_raises OK")
"###);
    assert_output(&out, r###"setrecursionlimit_zero_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/sys/unraisablehook_bare_exception_raises.py`.
#[test]
fn test_gen_errors_std_libs_sys_unraisablehook_bare_exception_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "unraisablehook_bare_exception_raises"
# subject = "sys.__unraisablehook__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__unraisablehook__: unraisablehook_bare_exception_raises (errors)."""
import sys

_raised = False
try:
    sys.__unraisablehook__(ValueError(42))
except TypeError:
    _raised = True
assert _raised, "unraisablehook_bare_exception_raises: expected TypeError"
print("unraisablehook_bare_exception_raises OK")
"###);
    assert_output(&out, r###"unraisablehook_bare_exception_raises OK
"###);
}
