# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_not_implemented_type_is_present"
# subject = "types.NotImplementedType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.NotImplementedType: api_not_implemented_type_is_present (surface)."""
import types

assert hasattr(types, "NotImplementedType")
print("api_not_implemented_type_is_present OK")
