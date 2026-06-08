# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "reduce_empty_no_initial_raises"
# subject = "functools.reduce"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: reduce_empty_no_initial_raises (errors)."""
import functools

_raised = False
try:
    functools.reduce(lambda a, b: a + b, [])
except TypeError:
    _raised = True
assert _raised, "reduce_empty_no_initial_raises: expected TypeError"
print("reduce_empty_no_initial_raises OK")
