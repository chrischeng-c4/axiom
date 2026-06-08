# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "implicit_concatenation_of_fstrings"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: adjacent f-string literals concatenate at compile time: f'part1' f'-part2' f'-part3' is 'part1-part2-part3'"""
# implicit string-literal concatenation works across f-string parts

long = (
    f"part1"
    f"-part2"
    f"-part3"
)
assert long == "part1-part2-part3", f"long = {long!r}"

print("implicit_concatenation_of_fstrings OK")
