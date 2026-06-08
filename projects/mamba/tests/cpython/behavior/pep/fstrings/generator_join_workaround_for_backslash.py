# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "fstrings"
# dimension = "behavior"
# case = "generator_join_workaround_for_backslash"
# subject = "fstring.expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.expression: binding a join result before the field avoids a literal backslash in the field: with items=[1,2,3], joined=', '.join(str(i) for i in items), f'[{joined}]' is '[1, 2, 3]'"""
# bind a computed value first, then interpolate it

items = [1, 2, 3]
joined = ", ".join(str(i) for i in items)
assert f"[{joined}]" == "[1, 2, 3]", f"join = {f'[{joined}]'!r}"

print("generator_join_workaround_for_backslash OK")
