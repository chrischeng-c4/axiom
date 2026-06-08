# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_builtin_function_type_is_present"
# subject = "types.BuiltinFunctionType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.BuiltinFunctionType: api_builtin_function_type_is_present (surface)."""
import types

assert hasattr(types, "BuiltinFunctionType")
print("api_builtin_function_type_is_present OK")
