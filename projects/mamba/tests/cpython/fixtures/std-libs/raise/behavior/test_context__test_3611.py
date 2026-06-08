# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_context__test_3611"
# subject = "cpython.test_raise.TestContext.test_3611"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestContext::test_3611 (CPython 3.12 oracle)."""

import gc
import sys


captured_unraisables = []
original_unraisablehook = sys.unraisablehook


def capture_unraisable(unraisable):
    captured_unraisables.append(unraisable)


class C:
    def __del__(self):
        try:
            1 / 0
        except Exception:
            raise


def trigger_del_reraise_context():
    x = C()
    try:
        try:
            trigger_del_reraise_context.x
        except AttributeError:
            del x
            gc.collect()
            raise TypeError
    except Exception as exc:
        assert isinstance(exc, TypeError), type(exc)
        assert exc.__context__ is not None
        assert isinstance(exc.__context__, AttributeError), exc.__context__


sys.unraisablehook = capture_unraisable
try:
    trigger_del_reraise_context()
finally:
    sys.unraisablehook = original_unraisablehook

assert len(captured_unraisables) == 1, captured_unraisables
assert captured_unraisables[0].exc_type is ZeroDivisionError, captured_unraisables[0]

print("TestContext::test_3611: ok")
