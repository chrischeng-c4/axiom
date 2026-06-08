# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_method_type_is_present"
# subject = "types.MethodType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.MethodType: api_method_type_is_present (surface)."""
import types

assert hasattr(types, "MethodType")
print("api_method_type_is_present OK")
