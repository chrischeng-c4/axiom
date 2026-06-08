# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_raising_callback_is_isolated"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: a callback that raises does not abort the run: the exception is reported to stderr and remaining callbacks still execute"""
import atexit
import contextlib
import io

order = []


def good():
    order.append("good")


def bad():
    raise ValueError("boom")


atexit._clear()
atexit.register(good)  # runs second (LIFO)
atexit.register(bad)   # runs first, raises
buf = io.StringIO()
with contextlib.redirect_stderr(buf):
    atexit._run_exitfuncs()
report = buf.getvalue()
assert order == ["good"], f"survivor still ran: {order}"
assert "ValueError" in report, f"exception reported to stderr: {report!r}"
atexit._clear()
print("run_raising_callback_is_isolated OK")
