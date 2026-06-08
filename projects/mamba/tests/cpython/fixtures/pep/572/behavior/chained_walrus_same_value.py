# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "chained_walrus_same_value"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: chained walrus (z := (y := (x := 0))) binds every name to the same innermost value 0"""
# Chained walrus binds every name to the same innermost value.
(z := (y := (x := 0)))
assert x == 0 and y == 0 and z == 0

print("chained_walrus_same_value OK")
