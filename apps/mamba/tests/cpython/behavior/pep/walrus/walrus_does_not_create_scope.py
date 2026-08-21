# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "walrus_does_not_create_scope"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus does not create a new scope: a walrus in an if-condition overwrites the existing module-level binding"""
# A walrus does not introduce a new scope; it overwrites the current binding.
outer = 99
if (outer := 42) > 0:
    pass
assert outer == 42, f"outer overwritten = {outer!r}"

print("walrus_does_not_create_scope OK")
