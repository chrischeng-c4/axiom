# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "real_imag_conjugate"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: as a real number a Fraction exposes .real == itself, .imag == 0, and conjugate() == itself"""
from fractions import Fraction

f = Fraction(3, 4)
assert f.real == f, "real part is the value itself"
assert f.imag == 0, "imag part is zero"
assert f.conjugate() == f, "conjugate of a real is itself"

print("real_imag_conjugate OK")
