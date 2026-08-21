# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "isclose_negative_rel_tol_raises_valueerror"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.isclose: isclose_negative_rel_tol_raises_valueerror (errors)."""
import cmath

_raised = False
try:
    cmath.isclose(1.0, 1.0, rel_tol=-1.0)
except ValueError:
    _raised = True
assert _raised, "isclose_negative_rel_tol_raises_valueerror: expected ValueError"
print("isclose_negative_rel_tol_raises_valueerror OK")
