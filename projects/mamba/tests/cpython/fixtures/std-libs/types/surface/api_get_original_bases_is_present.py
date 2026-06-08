# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_get_original_bases_is_present"
# subject = "types.get_original_bases"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.get_original_bases: api_get_original_bases_is_present (surface)."""
import types

assert hasattr(types, "get_original_bases")
print("api_get_original_bases_is_present OK")
