# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "exp_huge_raises_overflowerror"
# subject = "cmath.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.exp: exp_huge_raises_overflowerror (errors)."""
import cmath

_raised = False
try:
    cmath.exp(1e9)
except OverflowError:
    _raised = True
assert _raised, "exp_huge_raises_overflowerror: expected OverflowError"
print("exp_huge_raises_overflowerror OK")
