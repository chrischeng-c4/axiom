# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "lower_precedence_than_arithmetic"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: the walrus has lower precedence than arithmetic, so y = (x := 1) + 10 binds x to 1 and the whole expression to 11"""
# := binds 1, then the addition runs: the whole expression is 11.
x = 0
y = (x := 1) + 10
assert x == 1, f"x = {x!r}"
assert y == 11, f"y = {y!r}"

print("lower_precedence_than_arithmetic OK")
