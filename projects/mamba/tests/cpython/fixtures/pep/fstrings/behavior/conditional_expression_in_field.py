# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "conditional_expression_in_field"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a conditional expression evaluates inside a field: with n=5, f"{'odd' if n % 2 else 'even'}" is 'odd'"""
# a ternary conditional is a valid field expression

n = 5
assert f"{'odd' if n % 2 else 'even'}" == "odd", "conditional"

print("conditional_expression_in_field OK")
