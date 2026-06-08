# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "uniontype_is_a_type"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: uniontype_is_a_type (surface)."""
import types

assert type(types.UnionType).__name__ == "type"
print("uniontype_is_a_type OK")
