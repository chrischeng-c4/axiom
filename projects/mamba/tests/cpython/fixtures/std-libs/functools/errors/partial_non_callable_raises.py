# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "partial_non_callable_raises"
# subject = "functools.partial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial_non_callable_raises (errors)."""
import functools

_raised = False
try:
    functools.partial(42, 1)
except TypeError:
    _raised = True
assert _raised, "partial_non_callable_raises: expected TypeError"
print("partial_non_callable_raises OK")
