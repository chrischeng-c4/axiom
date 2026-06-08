# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_function_type_is_present"
# subject = "types.FunctionType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.FunctionType: api_function_type_is_present (surface)."""
import types

assert hasattr(types, "FunctionType")
print("api_function_type_is_present OK")
