# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "empty_and_whitespace_literals"
# subject = "fstring.literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.literal: an f-string with no fields is an ordinary literal: f'' is '', f' ' is ' ', f'a' is 'a'"""
# f-string literal portions behave like plain string literals

assert f"" == ""
assert f" " == " "
assert f"a" == "a"

print("empty_and_whitespace_literals OK")
