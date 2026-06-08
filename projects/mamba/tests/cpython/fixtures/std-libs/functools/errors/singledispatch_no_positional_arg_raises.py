# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "singledispatch_no_positional_arg_raises"
# subject = "functools.singledispatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.singledispatch: singledispatch_no_positional_arg_raises (errors)."""
import functools

_raised = False
try:
    functools.singledispatch(lambda *a: None)()
except TypeError:
    _raised = True
assert _raised, "singledispatch_no_positional_arg_raises: expected TypeError"
print("singledispatch_no_positional_arg_raises OK")
