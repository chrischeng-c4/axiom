# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "resolve_bases_is_callable"
# subject = "types.resolve_bases"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.resolve_bases: resolve_bases_is_callable (surface)."""
import types

assert callable(types.resolve_bases)
print("resolve_bases_is_callable OK")
