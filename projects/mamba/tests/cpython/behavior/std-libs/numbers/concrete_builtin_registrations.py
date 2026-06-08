# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "concrete_builtin_registrations"
# subject = "numbers.Integral"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: int registers as Integral, float as Real (not Integral), complex as Complex (not Real), bool as Integral, and str is not a Number"""
import numbers

# The built-in numeric types are registered at exactly their tower rung.
assert isinstance(1, numbers.Integral), "int is Integral"
assert isinstance(1.0, numbers.Real), "float is Real"
assert not isinstance(1.0, numbers.Integral), "float is NOT Integral"
assert isinstance(1j, numbers.Complex), "complex is Complex"
assert not isinstance(1j, numbers.Real), "complex is NOT Real"
assert isinstance(True, numbers.Integral), "bool is Integral (bool subclasses int)"

# A non-numeric type is not anywhere in the tower.
assert not isinstance("x", numbers.Number), "str is not a Number"

print("concrete_builtin_registrations OK")
