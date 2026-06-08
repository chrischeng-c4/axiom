# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "multiple_fields_with_arithmetic"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: multiple fields interleave with literal text and a derived field: with a=3, b=4, f'{a} + {b} = {a + b}' is '3 + 4 = 7'"""
# fields and literal runs concatenate left to right

a, b = 3, 4
assert f"{a} + {b} = {a + b}" == "3 + 4 = 7", "multiple"

print("multiple_fields_with_arithmetic OK")
