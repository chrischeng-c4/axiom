# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "list_comp_walrus_binds_enclosing_scope"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus target in a list comprehension binds in the ENCLOSING scope, not the comprehension's own scope (the deliberate PEP 572 exception): [(j := i) for i in range(5)] leaves j == 4"""
# A walrus target in a list comprehension binds in the ENCLOSING scope,
# not the comprehension's own scope (a deliberate PEP 572 exception).
res = [(j := i) for i in range(5)]
assert res == [0, 1, 2, 3, 4]
assert j == 4

print("list_comp_walrus_binds_enclosing_scope OK")
