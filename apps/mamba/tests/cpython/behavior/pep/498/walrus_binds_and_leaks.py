# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "walrus_binds_and_leaks"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: a walrus inside a field binds and leaks: f'{(z := 10)}' is '10' and z == 10 afterward"""
# the walrus operator works inside a replacement field

assert f"{(z := 10)}" == "10"
assert z == 10

print("walrus_binds_and_leaks OK")
