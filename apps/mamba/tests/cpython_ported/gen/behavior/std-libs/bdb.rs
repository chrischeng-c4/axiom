use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/bdb/bdbquit_subclasses_exception.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_bdbquit_subclasses_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "bdbquit_subclasses_exception"
# subject = "bdb.BdbQuit"
# kind = "semantic"
# xfail = "mamba bdb stub: BdbQuit is not a real Exception subclass (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.BdbQuit: BdbQuit is an Exception subclass usable for cleanly aborting a debug session"""
import bdb

assert issubclass(bdb.BdbQuit, Exception), "BdbQuit is an Exception subclass"

_caught = False
try:
    raise bdb.BdbQuit
except bdb.BdbQuit:
    _caught = True
assert _caught, "BdbQuit is raisable and catchable"

print("bdbquit_subclasses_exception OK")
"###);
    assert_output(&out, r###"bdbquit_subclasses_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/breakpoint_bplist_initially_empty.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_breakpoint_bplist_initially_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "breakpoint_bplist_initially_empty"
# subject = "bdb.Breakpoint"
# kind = "semantic"
# xfail = "mamba bdb stub: Breakpoint.bplist is None, not an empty dict (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Breakpoint: the class-level Breakpoint.bplist registry is an empty dict before any breakpoint is created"""
import bdb

assert isinstance(bdb.Breakpoint.bplist, dict), f"bplist type = {type(bdb.Breakpoint.bplist)!r}"
assert bdb.Breakpoint.bplist == {}, f"bplist not empty: {bdb.Breakpoint.bplist!r}"

print("breakpoint_bplist_initially_empty OK")
"###);
    assert_output(&out, r###"breakpoint_bplist_initially_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/clear_break_no_breakpoints_returns_error_string.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_clear_break_no_breakpoints_returns_error_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "clear_break_no_breakpoints_returns_error_string"
# subject = "bdb.Bdb.clear_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no clear_break method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.clear_break: clear_break where no breakpoint exists returns an error string rather than raising"""
import bdb

_d = bdb.Bdb()
_err = _d.clear_break("/some/file.py", 999)
assert isinstance(_err, str), f"clear_break invalid returns a str, got {_err!r}"
assert _err, "error message is non-empty"

print("clear_break_no_breakpoints_returns_error_string OK")
"###);
    assert_output(&out, r###"clear_break_no_breakpoints_returns_error_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/get_break_missing_returns_false.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_get_break_missing_returns_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "get_break_missing_returns_false"
# subject = "bdb.Bdb.get_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no get_break method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.get_break: get_break for a file/line with no breakpoint returns False (no raise)"""
import bdb

_d = bdb.Bdb()
_res = _d.get_break("no_such_file.py", 1)
assert _res is False, f"get_break missing = {_res!r}"

print("get_break_missing_returns_false OK")
"###);
    assert_output(&out, r###"get_break_missing_returns_false OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/runcall_passes_arguments.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_runcall_passes_arguments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "runcall_passes_arguments"
# subject = "bdb.Bdb.runcall"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runcall: runcall forwards positional arguments to the traced function: runcall(add, 10, 20) returns 30"""
import bdb


def _add(a, b):
    return a + b


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_continue()


_d = _Dbg()
_r = _d.runcall(_add, 10, 20)
assert _r == 30, f"runcall with args = {_r!r}"

print("runcall_passes_arguments OK")
"###);
    assert_output(&out, r###"runcall_passes_arguments OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/runcall_returns_function_result.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_runcall_returns_function_result() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "runcall_returns_function_result"
# subject = "bdb.Bdb.runcall"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runcall: runcall runs a function under the debugger and returns its result (a Bdb subclass that set_continue()s on each line returns 42 from a lambda)"""
import bdb


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_continue()


_d = _Dbg()
_r = _d.runcall(lambda: 42)
assert _r == 42, f"runcall result = {_r!r}"

print("runcall_returns_function_result OK")
"###);
    assert_output(&out, r###"runcall_returns_function_result OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/runeval_evaluates_expression.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_runeval_evaluates_expression() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "runeval_evaluates_expression"
# subject = "bdb.Bdb.runeval"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runeval method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runeval: runeval evaluates an expression under the debugger and returns its value: runeval('1 + 2 + 3') is 6"""
import bdb


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_continue()


_d = _Dbg()
_r = _d.runeval("1 + 2 + 3")
assert _r == 6, f"runeval = {_r!r}"

print("runeval_evaluates_expression OK")
"###);
    assert_output(&out, r###"runeval_evaluates_expression OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/set_break_clear_all_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_set_break_clear_all_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "set_break_clear_all_roundtrip"
# subject = "bdb.Bdb.set_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no breaks/set_break/clear_all_breaks (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.set_break: breaks starts empty, set_break on a real source line populates it, and clear_all_breaks empties it again"""
import bdb
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _src = os.path.join(_td, "module_under_debug.py")
    with open(_src, "w", encoding="utf-8") as _f:
        _f.write("def g():\n    return 1\n")

    _d = bdb.Bdb()
    assert _d.breaks == {}, "breaks empty initially"
    _err = _d.set_break(_src, 2)
    assert _err is None, f"set_break on a real line returns None, got {_err!r}"
    assert len(_d.breaks) > 0, "break added to breaks dict"
    _d.clear_all_breaks()
    assert _d.breaks == {}, "breaks cleared"

print("set_break_clear_all_roundtrip OK")
"###);
    assert_output(&out, r###"set_break_clear_all_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/set_break_invalid_file_returns_error_string.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_set_break_invalid_file_returns_error_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "set_break_invalid_file_returns_error_string"
# subject = "bdb.Bdb.set_break"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no set_break method (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.set_break: set_break on a nonexistent file returns an error string rather than raising"""
import bdb

_d = bdb.Bdb()
_err = _d.set_break("/no/such/file.py", 1)
assert isinstance(_err, str), f"set_break invalid returns a str, got {_err!r}"
assert _err, "error message is non-empty"

print("set_break_invalid_file_returns_error_string OK")
"###);
    assert_output(&out, r###"set_break_invalid_file_returns_error_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/set_quit_aborts_and_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_set_quit_aborts_and_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "set_quit_aborts_and_returns_none"
# subject = "bdb.Bdb.set_quit"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall/set_quit (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.set_quit: set_quit() during tracing aborts the run (runcall returns None) and sets the quitting flag True"""
import bdb


class _Dbg(bdb.Bdb):
    def user_line(self, frame):
        self.set_quit()


def _long_fn():
    a = 1
    b = 2
    c = 3
    return a + b + c


_d = _Dbg()
_r = _d.runcall(_long_fn)
assert _r is None, f"runcall returns None on set_quit: {_r!r}"
assert _d.quitting is True, "quitting flag set after set_quit"

print("set_quit_aborts_and_returns_none OK")
"###);
    assert_output(&out, r###"set_quit_aborts_and_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/skip_default_is_none.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_skip_default_is_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "skip_default_is_none"
# subject = "bdb.Bdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb: a freshly constructed Bdb has skip == None (no skip patterns configured)"""
import bdb

_d = bdb.Bdb()
assert _d.skip is None, f"default skip = {_d.skip!r}"

print("skip_default_is_none OK")
"###);
    assert_output(&out, r###"skip_default_is_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/user_line_called_per_line.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_user_line_called_per_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "user_line_called_per_line"
# subject = "bdb.Bdb.user_line"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall/user_line tracing (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.user_line: the debugger invokes user_line at least once while tracing a multi-statement function, and runcall still returns the correct result"""
import bdb


class _Dbg(bdb.Bdb):
    def __init__(self):
        super().__init__()
        self.line_count = 0

    def user_line(self, frame):
        self.line_count += 1
        if self.line_count >= 5:
            self.set_continue()


def _simple():
    x = 1
    y = x + 1
    z = y + 1
    return z


_d = _Dbg()
_r = _d.runcall(_simple)
assert _r == 3, f"_simple result = {_r!r}"
assert _d.line_count > 0, "user_line was called"

print("user_line_called_per_line OK")
"###);
    assert_output(&out, r###"user_line_called_per_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/bdb/user_return_receives_retval.py`.
#[test]
fn test_gen_behavior_std_libs_bdb_user_return_receives_retval() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "user_return_receives_retval"
# subject = "bdb.Bdb.user_return"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb has no runcall/user_return tracing (#1261)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.user_return: user_return fires when a traced function returns and receives the return value (the recorded returns list contains the function's result)"""
import bdb


class _Dbg(bdb.Bdb):
    def __init__(self):
        super().__init__()
        self.returns = []

    def user_line(self, frame):
        self.set_step()

    def user_return(self, frame, retval):
        self.returns.append(retval)
        self.set_continue()


def _fn():
    return "result"


_d = _Dbg()
_r = _d.runcall(_fn)
assert _r == "result", f"runcall = {_r!r}"
assert "result" in _d.returns, f"user_return called: {_d.returns!r}"

print("user_return_receives_retval OK")
"###);
    assert_output(&out, r###"user_return_receives_retval OK
"###);
}
