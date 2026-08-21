# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "limit_denominator_best_approximation"
# subject = "fractions.Fraction.limit_denominator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction.limit_denominator: limit_denominator caps the denominator while finding the closest value: 355/113 limited to <=10 has denom <=10, and pi captured to 1000 gives 355/113"""
from fractions import Fraction

_pi = Fraction(355, 113)  # a close rational approximation of pi
_pi10 = _pi.limit_denominator(10)
assert _pi10.denominator <= 10, f"pi limited denom = {_pi10.denominator!r}"
# limit_denominator finds the closest value within the bound.
_pi1000 = Fraction(3.141592653589793).limit_denominator(1000)
assert _pi1000 == Fraction(355, 113), f"pi to 1000 = {_pi1000!r}"

print("limit_denominator_best_approximation OK")
