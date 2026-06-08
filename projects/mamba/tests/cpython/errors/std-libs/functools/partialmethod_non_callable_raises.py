# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "partialmethod_non_callable_raises"
# subject = "functools.partialmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partialmethod: partialmethod_non_callable_raises (errors)."""
import functools

_raised = False
try:
    type("Bad", (), {"m": functools.partialmethod(None, 1)})
except TypeError:
    _raised = True
assert _raised, "partialmethod_non_callable_raises: expected TypeError"
print("partialmethod_non_callable_raises OK")
