# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "nested_comp_walrus_leaks_enclosing"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: the enclosing-scope walrus binding survives nesting; the inner walrus of a nested comprehension leaks out to the surrounding scope"""
# The enclosing binding survives nesting; the inner walrus leaks out.
nested = [[(spam := i) for i in range(3)] for _ in range(2)]
assert nested == [[0, 1, 2], [0, 1, 2]]
assert spam == 2

print("nested_comp_walrus_leaks_enclosing OK")
