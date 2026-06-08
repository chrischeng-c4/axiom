# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "real_world"
# case = "tracing_debugger_session_walkthrough"
# subject = "bdb.Bdb"
# kind = "semantic"
# xfail = "mamba bdb stub: Bdb() is dict-like, no debugger methods (#1261)"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
"""bdb.Bdb: a downstream consumer subclasses Bdb to drive a real trace session: runcall a function with args, count user_line / user_return callbacks, capture the return value, and set/clear a breakpoint on a temp source file"""
import bdb
import os
import tempfile


class _TraceDriver(bdb.Bdb):
    def __init__(self):
        super().__init__()
        self.line_hits = 0
        self.return_values = []

    def user_line(self, frame):
        self.line_hits += 1
        self.set_step()

    def user_return(self, frame, retval):
        self.return_values.append(retval)
        self.set_continue()


def _compute(a, b):
    total = a + b
    doubled = total * 2
    return doubled


# 1. Drive a function under the debugger with arguments and callbacks.
_drv = _TraceDriver()
_result = _drv.runcall(_compute, 3, 4)
assert _result == 14, f"runcall result = {_result!r}"
assert _drv.line_hits > 0, "user_line fired during tracing"
assert _drv.return_values == [14], f"user_return captured = {_drv.return_values!r}"

# 2. Manage breakpoints against a real source file on disk.
with tempfile.TemporaryDirectory() as _td:
    _src = os.path.join(_td, "target.py")
    with open(_src, "w", encoding="utf-8") as _f:
        _f.write("def h():\n    return 99\n")

    _dbg = bdb.Bdb()
    assert _dbg.breaks == {}, "no breakpoints initially"
    _err = _dbg.set_break(_src, 2)
    assert _err is None, f"set_break on a real line returns None, got {_err!r}"
    assert len(_dbg.breaks) > 0, "breakpoint registered"
    assert _dbg.get_break(_src, 2) is True, "get_break sees the registered line"
    _cleared = _dbg.clear_break(_src, 2)
    assert _cleared is None, f"clear_break of a real breakpoint returns None, got {_cleared!r}"
    _dbg.clear_all_breaks()
    assert _dbg.breaks == {}, "all breakpoints cleared"

print("tracing_debugger_session_walkthrough OK")
