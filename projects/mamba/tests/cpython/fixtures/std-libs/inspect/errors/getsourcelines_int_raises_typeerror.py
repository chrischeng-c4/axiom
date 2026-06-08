# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getsourcelines_int_raises_typeerror"
# subject = "inspect.getsourcelines"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getsourcelines: getsourcelines_int_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getsourcelines(int)
except TypeError:
    _raised = True
assert _raised, "getsourcelines_int_raises_typeerror: expected TypeError"
print("getsourcelines_int_raises_typeerror OK")
