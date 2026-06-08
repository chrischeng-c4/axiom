# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "hash_matches_equal_values"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: hash agrees with == across numeric types so Fractions key dicts correctly: hash(Fraction(6,3)) == hash(2) == hash(2.0)"""
from fractions import Fraction

assert hash(Fraction(6, 3)) == hash(2), "hash matches equal int"
assert hash(Fraction(6, 3)) == hash(2.0), "hash matches equal float"
assert hash(Fraction(1, 2)) == hash(0.5), "hash matches equal 0.5"
# Equal values therefore collapse to one dict key.
assert len({Fraction(6, 3): "a", 2: "b"}) == 1, "equal Fraction/int share a key"

print("hash_matches_equal_values OK")
