# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isclose_abs_tol_near_and_far"
# subject = "cmath.isclose"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isclose: isclose with abs_tol: nearby values are close, far-apart values are not"""
import cmath

assert cmath.isclose(1j, 1.0001j, abs_tol=0.001), "isclose abs_tol near"
assert not cmath.isclose(1j, 2j, abs_tol=0.001), "not isclose far apart"
print("isclose_abs_tol_near_and_far OK")
