# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "reduce_non_iterable_raises"
# subject = "functools.reduce"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: reduce_non_iterable_raises (errors)."""
import functools

_raised = False
try:
    functools.reduce(lambda a, b: a + b, 123)
except TypeError:
    _raised = True
assert _raised, "reduce_non_iterable_raises: expected TypeError"
print("reduce_non_iterable_raises OK")
