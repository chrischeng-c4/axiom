# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_stopiteration_passes_through"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: PEP 479: a StopIteration raised inside the with-body is NOT swallowed or replaced by the manager — the same StopIteration instance propagates unchanged"""
import contextlib

stop = StopIteration("spam")


@contextlib.contextmanager
def passthrough():
    yield


_raised = False
try:
    with passthrough():
        raise stop
except StopIteration as e:
    _raised = True
    assert e is stop, "the same StopIteration instance must propagate unchanged"
assert _raised, "expected StopIteration to pass through the manager"

print("contextmanager_stopiteration_passes_through OK")
