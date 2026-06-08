# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "named_expr_value_usable_inline"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a named expression evaluates to the assigned value usable inline: len((lines := [1,2,3])) is 3 and lines is bound to the list"""
# A named expression evaluates to the assigned value, usable inline.
total = len((lines := [1, 2, 3]))
assert total == 3
assert lines == [1, 2, 3]

print("named_expr_value_usable_inline OK")
