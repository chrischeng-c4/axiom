# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "construct_from_float_exact"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction(float) captures the exact binary value: Fraction(0.5) == 1/2, while Fraction(0.1) has a large denominator that limit_denominator(100) tidies back to 1/10"""
from fractions import Fraction

assert Fraction(0.5) == Fraction(1, 2), "0.5 is exactly representable"
_tenth = Fraction(0.1)
# 0.1 is not exactly representable in binary: the denominator is large.
assert _tenth.denominator > 1, f"0.1 float denom = {_tenth.denominator!r}"
assert _tenth.limit_denominator(100) == Fraction(1, 10), "limit 0.1 -> 1/10"

print("construct_from_float_exact OK")
