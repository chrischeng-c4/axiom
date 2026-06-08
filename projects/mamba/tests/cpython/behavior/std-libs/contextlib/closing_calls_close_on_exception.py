# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "closing_calls_close_on_exception"
# subject = "contextlib.closing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.closing: contextlib.closing(obj) still calls obj.close() when the with-body raises, before re-raising the exception"""
import contextlib


class Resource:
    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True


r = Resource()
_propagated = False
try:
    with contextlib.closing(r):
        raise ZeroDivisionError("boom")
except ZeroDivisionError:
    _propagated = True
assert r.closed, "closing must call close() even when the body raises"
assert _propagated, "the body exception must still propagate"

print("closing_calls_close_on_exception OK")
