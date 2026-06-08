# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_finally_runs_on_exception"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager whose body raises still runs its finally-clause cleanup before the exception propagates out of the with-block"""
import contextlib

cleaned = False


@contextlib.contextmanager
def cm():
    global cleaned
    try:
        yield
    finally:
        cleaned = True


_propagated = False
try:
    with cm():
        raise RuntimeError("boom")
except RuntimeError:
    _propagated = True
assert cleaned, "finally cleanup must run when the body raises"
assert _propagated, "the original exception must still propagate"

print("contextmanager_finally_runs_on_exception OK")
