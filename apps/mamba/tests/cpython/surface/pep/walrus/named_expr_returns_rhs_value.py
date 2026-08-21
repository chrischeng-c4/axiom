# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "surface"
# case = "named_expr_returns_rhs_value"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a parenthesized named expression evaluates to the right-hand value and binds the name: x = (y := 42) leaves both x and y equal to 42"""
# := returns the right-hand value and binds the inner name.
x = (y := 42)
assert y == 42, f"y = {y!r}"
assert x == 42, f"x = {x!r}"

print("named_expr_returns_rhs_value OK")
