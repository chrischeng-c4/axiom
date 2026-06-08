# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isfinite_only_for_finite"
# subject = "cmath.isfinite"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isfinite: isfinite is True for a finite complex and False once a component is infinite"""
import cmath

assert cmath.isfinite(complex(1, 2)), "isfinite(1+2j)"
assert not cmath.isfinite(complex(float("inf"), 0)), "not isfinite inf"
print("isfinite_only_for_finite OK")
