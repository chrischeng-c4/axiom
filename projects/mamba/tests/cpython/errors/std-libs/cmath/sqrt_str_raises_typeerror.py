# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "sqrt_str_raises_typeerror"
# subject = "cmath.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.sqrt: sqrt_str_raises_typeerror (errors)."""
import cmath

_raised = False
try:
    cmath.sqrt("hello")
except TypeError:
    _raised = True
assert _raised, "sqrt_str_raises_typeerror: expected TypeError"
print("sqrt_str_raises_typeerror OK")
