# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "abs_zero_is_zero"
# subject = "abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs: abs() of complex zero (and signed zero) is 0.0"""
import cmath  # noqa: F401

assert abs(complex(0.0, 0.0)) == 0.0, "abs of zero is zero"
assert abs(complex(-0.0, -0.0)) == 0.0, "abs of signed zero is zero"
print("abs_zero_is_zero OK")
