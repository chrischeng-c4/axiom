# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "runsource_exec_error_caught_to_stderr"
# subject = "code.InteractiveInterpreter.runsource"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveInterpreter.runsource: an exception from executed source is caught (not propagated): runsource('1 / 0') returns False and the traceback (containing 'ZeroDivisionError') is written to stderr via showtraceback"""
import code
import io
import contextlib

_interp = code.InteractiveInterpreter({})
_buf = io.StringIO()
with contextlib.redirect_stderr(_buf):
    _res = _interp.runsource("1 / 0")
# The runtime error is caught and reported, not raised; runsource still returns
# False (the source was complete) and the traceback is on stderr.
assert _res is False, f"complete-but-erroring source -> False, got {_res!r}"
assert "ZeroDivisionError" in _buf.getvalue(), "ZeroDivisionError traceback on stderr"

print("runsource_exec_error_caught_to_stderr OK")
