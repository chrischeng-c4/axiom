# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "is_immutable_slots"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: Fraction uses __slots__: instances reject arbitrary attribute assignment, numerator/denominator are read-only, and there is no instance __dict__"""
from fractions import Fraction

r = Fraction(13, 7)

# Arbitrary attribute assignment is rejected (no instance __dict__).
_raised = False
try:
    r.extra = 10
except AttributeError:
    _raised = True
assert _raised, "setting an unknown attribute raises AttributeError"

# numerator / denominator are read-only properties.
_ro = False
try:
    r.numerator = 1
except AttributeError:
    _ro = True
assert _ro, "numerator is read-only"

# Instances expose no __dict__ thanks to __slots__.
assert not hasattr(r, "__dict__"), "Fraction instance has no __dict__"

print("is_immutable_slots OK")
