# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "multiple_fields_mix_with_literal_text"
# subject = "fstring.interpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.interpolation: multiple fields interleave with literal text: with a=98, b='abc', f'X{a}Y{b}Z' is 'X98YabcZ'"""
# fields and literal runs concatenate left to right

a, b = 98, "abc"
assert f"X{a}Y{b}Z" == "X98YabcZ"

print("multiple_fields_mix_with_literal_text OK")
