# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "abs_infinite_component_is_inf"
# subject = "abs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abs: abs() of any complex with an infinite component is +inf, and an infinite component beats a NaN component"""
import math

INF = float("inf")
NAN = float("nan")

assert abs(complex(INF, 2.3)) == INF, "abs with real inf is inf"
assert abs(complex(2.3, -INF)) == INF, "abs with imag -inf is inf"
assert abs(complex(-INF, -INF)) == INF, "abs of inf+inf is inf"
assert abs(complex(NAN, -INF)) == INF, "inf beats nan in abs"
assert abs(complex(INF, NAN)) == INF, "inf beats nan in abs (real side)"
print("abs_infinite_component_is_inf OK")
