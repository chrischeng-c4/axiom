# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "tower_abcs_are_callable"
# subject = "numbers.Number"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Number: all five tower ABCs (Number/Complex/Real/Rational/Integral) are callable class objects, not function-name stubs (#1261)"""
import numbers

# Regression guard for #1261: the five ABC entries must be real callable class
# objects, not function-name string stubs that AttributeError on reference.
for abc in (numbers.Number, numbers.Complex, numbers.Real,
            numbers.Rational, numbers.Integral):
    assert callable(abc), abc

# They are distinct names along the tower, not aliases of one stub.
names = {numbers.Number.__name__, numbers.Complex.__name__, numbers.Real.__name__,
         numbers.Rational.__name__, numbers.Integral.__name__}
assert names == {"Number", "Complex", "Real", "Rational", "Integral"}, names

print("tower_abcs_are_callable OK")
