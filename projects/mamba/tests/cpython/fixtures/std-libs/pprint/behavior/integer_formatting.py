# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "integer_formatting"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: plain ints render as decimal repr; underscore_numbers=True groups thousands (1_234_567); an int subclass is rendered via its own __repr__"""
import pprint

# Plain integers render as their decimal repr.
assert pprint.pformat(1234567) == "1234567"

# underscore_numbers=True groups digits in threes.
assert pprint.pformat(1234567, underscore_numbers=True) == "1_234_567"
assert pprint.pformat(1000, underscore_numbers=True) == "1_000"
assert pprint.pformat(999, underscore_numbers=True) == "999"


# An int subclass with a custom __repr__ is honored by pformat.
class Temperature(int):
    def __new__(cls, celsius):
        return super().__new__(cls, celsius)

    def __repr__(self):
        return f"{self + 273.15}K"


assert pprint.pformat(Temperature(1000)) == "1273.15K"
print("integer_formatting OK")
