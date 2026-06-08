# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "inline_arithmetic_expression"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: an arithmetic expression evaluates inline: with x=10, f'{x + 5}' is '15'"""
# replacement fields hold arbitrary expressions

x = 10
assert f"{x + 5}" == "15", f"arith = {f'{x + 5}'!r}"

print("inline_arithmetic_expression OK")
