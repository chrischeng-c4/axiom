# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "nested_fstring_is_just_an_expression"
# subject = "fstring.nesting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.nesting: an inner f-string is just another expression: with y=5, f"{f'{y}' * 3}" is '555'"""
# f-strings nest because a field holds any expression

y = 5
assert f"{f'{y}' * 3}" == "555"

print("nested_fstring_is_just_an_expression OK")
