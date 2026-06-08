# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_no_yield_raises_runtimeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager-decorated function that never yields raises RuntimeError when its with-block is entered (the inner generator stops immediately)"""
import contextlib


# A real generator (it contains a `yield`) that returns before ever reaching
# the yield. contextmanager.__enter__ calls next() and the generator stops
# immediately -> RuntimeError("generator didn't yield").
@contextlib.contextmanager
def no_yield():
    if False:
        yield  # makes this a generator without ever yielding
    return


_raised = False
try:
    with no_yield():
        pass
except RuntimeError as e:
    _raised = True
    assert "didn't yield" in str(e), str(e)
assert _raised, "expected RuntimeError when generator never yields"

print("contextmanager_no_yield_raises_runtimeerror OK")
