# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "tower_inheritance_chain"
# subject = "numbers.Integral"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: the numeric tower nests Integral < Rational < Real < Complex < Number via __bases__, issubclass, and the Integral MRO"""
import numbers

# Each rung's direct base is the next one up the tower (recovered from the
# errors.py integral_bases probe, generalized to the whole chain).
assert [b.__name__ for b in numbers.Integral.__bases__] == ["Rational"]
assert [b.__name__ for b in numbers.Rational.__bases__] == ["Real"]
assert [b.__name__ for b in numbers.Real.__bases__] == ["Complex"]
assert [b.__name__ for b in numbers.Complex.__bases__] == ["Number"]

# issubclass reflects the full nesting.
assert issubclass(numbers.Integral, numbers.Rational)
assert issubclass(numbers.Rational, numbers.Real)
assert issubclass(numbers.Real, numbers.Complex)
assert issubclass(numbers.Complex, numbers.Number)
assert issubclass(int, numbers.Integral)

# The Integral MRO walks the whole tower down to object.
assert [c.__name__ for c in numbers.Integral.__mro__] == [
    "Integral", "Rational", "Real", "Complex", "Number", "object",
], [c.__name__ for c in numbers.Integral.__mro__]

print("tower_inheritance_chain OK")
