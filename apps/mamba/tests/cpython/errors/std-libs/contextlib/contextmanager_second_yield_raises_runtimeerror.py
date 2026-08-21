# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_second_yield_raises_runtimeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager generator that yields a second time raises RuntimeError at __exit__ (the manager forbids resuming past the single yield point)"""
import contextlib


@contextlib.contextmanager
def two_yields():
    yield 1
    yield 2  # resuming the generator past the single yield is illegal


_raised = False
try:
    with two_yields():
        pass
except RuntimeError as e:
    _raised = True
    assert "didn't stop" in str(e), str(e)
assert _raised, "expected RuntimeError when generator yields a second time"

print("contextmanager_second_yield_raises_runtimeerror OK")
