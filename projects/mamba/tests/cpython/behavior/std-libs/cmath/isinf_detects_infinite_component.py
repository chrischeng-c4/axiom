# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isinf_detects_infinite_component"
# subject = "cmath.isinf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isinf: isinf is True when either the real or imaginary component is infinite, and accepts plain int/float too"""
import cmath

assert cmath.isinf(complex(float("inf"), 0)), "isinf real inf"
assert cmath.isinf(complex(0, float("inf"))), "isinf imag inf"
assert cmath.isinf(float("inf")), "isinf(float inf)"
# Finite ints and the finite imaginary unit are not infinite.
assert not cmath.isinf(1), "isinf(int 1) is False"
assert not cmath.isinf(1j), "isinf(1j) is False"
print("isinf_detects_infinite_component OK")
