# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isclose_rel_tol_near"
# subject = "cmath.isclose"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isclose: isclose with rel_tol accepts two nearly-equal complex values"""
import cmath

assert cmath.isclose(1 + 1j, 1 + 1.0000001j, rel_tol=1e-5), "isclose rel_tol near"
print("isclose_rel_tol_near OK")
