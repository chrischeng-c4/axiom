# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "builtin_call_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a builtin call result interpolates: with data=[10,20,30], f'sum={sum(data)}' is 'sum=60'"""
# a builtin call is a valid field expression

data = [10, 20, 30]
total_str = f"sum={sum(data)}"
assert total_str == "sum=60", f"sum = {total_str!r}"

print("builtin_call_in_field OK")
