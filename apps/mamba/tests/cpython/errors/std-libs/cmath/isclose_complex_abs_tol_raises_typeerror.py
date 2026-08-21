# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "isclose_complex_abs_tol_raises_typeerror"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.isclose: isclose_complex_abs_tol_raises_typeerror (errors)."""
import cmath

_raised = False
try:
    cmath.isclose(1j, 1j, abs_tol=1j)
except TypeError:
    _raised = True
assert _raised, "isclose_complex_abs_tol_raises_typeerror: expected TypeError"
print("isclose_complex_abs_tol_raises_typeerror OK")
