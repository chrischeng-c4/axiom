# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "walrus_in_while_condition_reevaluated"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a while-condition is re-evaluated each iteration; an integer floor-sqrt loop over 9 converges to base == 1"""
# Walrus in a while-condition is re-evaluated each iteration; here it
# computes the integer floor square root of 9.
base, root, target = 9, 2, 3
while base > (step := (target // base ** (root - 1))):
    base = ((root - 1) * base + step) // root
assert base == 1

print("walrus_in_while_condition_reevaluated OK")
