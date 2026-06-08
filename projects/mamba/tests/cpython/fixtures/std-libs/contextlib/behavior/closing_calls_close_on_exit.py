# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "closing_calls_close_on_exit"
# subject = "contextlib.closing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.closing: contextlib.closing(obj) calls obj.close() when the with-block exits normally, and obj is not closed before the block ends"""
import contextlib


class Resource:
    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True


r = Resource()
with contextlib.closing(r) as entered:
    assert entered is r, "closing yields the wrapped object"
    assert not r.closed, "must not be closed inside the block"
assert r.closed, "closing must call close() on normal exit"

print("closing_calls_close_on_exit OK")
