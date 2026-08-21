# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""inspect.stack/getouterframes over real f_back chains (CPython 3.12 oracle)."""

import inspect
import sys


def leaf():
    frame = inspect.currentframe()
    outer = inspect.getouterframes(frame)
    stack = inspect.stack()

    assert len(outer) >= 3
    assert len(stack) >= 3
    assert outer[0].function == "leaf"
    assert stack[0].function == "leaf"
    assert any(info.function == "caller" for info in outer)
    assert any(info.function == "<module>" for info in outer)
    assert frame.f_back.f_code.co_name == "caller"

    first = outer[0]
    assert first[0] is first.frame
    assert first[1] == first.filename
    assert first[2] == first.lineno
    assert first[3] == first.function

    try:
        sys._getframe(9999)
        raise AssertionError("expected ValueError for too-deep _getframe")
    except ValueError:
        pass


def caller():
    leaf()


caller()
print("inspect_stack_outerframes OK")
