# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_resolve_bases_is_present"
# subject = "types.resolve_bases"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.resolve_bases: api_resolve_bases_is_present (surface)."""
import types

assert hasattr(types, "resolve_bases")
print("api_resolve_bases_is_present OK")
