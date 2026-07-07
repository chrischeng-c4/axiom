# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""inspect.currentframe().f_lineno advances at each statement (CPython 3.12 oracle)."""

import inspect


def line_probe():
    first = inspect.currentframe().f_lineno
    second = inspect.currentframe().f_lineno
    return first, second


first, second = line_probe()
assert second > first, (first, second)
print("currentframe_lineno_advances OK", first, second)
