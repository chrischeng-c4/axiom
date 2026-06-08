# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "bare_named_expr_binds"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a bare parenthesized named expression (a := 10) binds the name and the statement's value is the assigned value"""
# A bare named expression binds the name and the statement is the value.
(a := 10)
assert a == 10

print("bare_named_expr_binds OK")
