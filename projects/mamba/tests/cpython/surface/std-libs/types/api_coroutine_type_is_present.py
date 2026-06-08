# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_coroutine_type_is_present"
# subject = "types.CoroutineType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.CoroutineType: api_coroutine_type_is_present (surface)."""
import types

assert hasattr(types, "CoroutineType")
print("api_coroutine_type_is_present OK")
