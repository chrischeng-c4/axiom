# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "572"
# dimension = "behavior"
# case = "slice_with_walrus_index_allowed"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a parenthesized walrus inside a slice bound is allowed: s[(i := 0):] compiles, runs, and binds i == 0"""
# A parenthesized walrus inside a slice bound is allowed.
s = "abc"
sliced = s[(i := 0):]
assert i == 0
assert sliced == "abc"

print("slice_with_walrus_index_allowed OK")
