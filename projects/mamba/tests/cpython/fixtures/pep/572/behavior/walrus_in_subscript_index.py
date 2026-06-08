# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "walrus_in_subscript_index"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus inside a subscript index binds and indexes in one expression: data[(pos := 2)] binds pos == 2 and picks data[2]"""
# Walrus inside a subscript index binds and indexes in one expression.
data = [10, 20, 30]
picked = data[(pos := 2)]
assert pos == 2
assert picked == 30

print("walrus_in_subscript_index OK")
