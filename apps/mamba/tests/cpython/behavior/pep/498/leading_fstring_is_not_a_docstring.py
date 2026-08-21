# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "leading_fstring_is_not_a_docstring"
# subject = "fstring.statement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.statement: an f-string as a function's first statement is not a docstring: fn.__doc__ is None"""
# only a plain str literal as the first statement is a docstring

def fn():
    f"not a docstring"

assert fn.__doc__ is None

print("leading_fstring_is_not_a_docstring OK")
