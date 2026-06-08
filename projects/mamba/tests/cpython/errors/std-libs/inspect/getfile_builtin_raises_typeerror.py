# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getfile_builtin_raises_typeerror"
# subject = "inspect.getfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getfile: getfile_builtin_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getfile(len)
except TypeError:
    _raised = True
assert _raised, "getfile_builtin_raises_typeerror: expected TypeError"
print("getfile_builtin_raises_typeerror OK")
