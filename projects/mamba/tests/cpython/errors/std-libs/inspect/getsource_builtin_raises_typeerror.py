# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getsource_builtin_raises_typeerror"
# subject = "inspect.getsource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getsource: getsource_builtin_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getsource(len)
except TypeError:
    _raised = True
assert _raised, "getsource_builtin_raises_typeerror: expected TypeError"
print("getsource_builtin_raises_typeerror OK")
