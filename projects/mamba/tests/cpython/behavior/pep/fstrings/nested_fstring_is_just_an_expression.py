# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "nested_fstring_is_just_an_expression"
# subject = "fstring.nesting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.nesting: an inner f-string is just another expression in a field: inner = f"{'world'}" then f'hello {inner}' is 'hello world'"""
# f-strings nest because a field holds any expression

inner = f"{'world'}"
assert f"hello {inner}" == "hello world", f"nested f = {f'hello {inner}'!r}"

print("nested_fstring_is_just_an_expression OK")
