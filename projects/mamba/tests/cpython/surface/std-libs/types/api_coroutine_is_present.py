# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_coroutine_is_present"
# subject = "types.coroutine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.coroutine: api_coroutine_is_present (surface)."""
import types

assert hasattr(types, "coroutine")
print("api_coroutine_is_present OK")
