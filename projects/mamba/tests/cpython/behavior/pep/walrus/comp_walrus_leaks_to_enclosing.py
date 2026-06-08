# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "comp_walrus_leaks_to_enclosing"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus target inside a comprehension leaks to the enclosing scope, holding the last value assigned during iteration"""
# A comprehension walrus target leaks out, holding the last assigned value.
leak = None
lst = [leak := v for v in range(5)]
assert lst == [0, 1, 2, 3, 4], f"lst = {lst!r}"
assert leak == 4, f"leak = {leak!r}"

print("comp_walrus_leaks_to_enclosing OK")
