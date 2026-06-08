# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getmro_int_raises_attributeerror"
# subject = "inspect.getmro"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getmro: getmro_int_raises_attributeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getmro(42)
except AttributeError:
    _raised = True
assert _raised, "getmro_int_raises_attributeerror: expected AttributeError"
print("getmro_int_raises_attributeerror OK")
