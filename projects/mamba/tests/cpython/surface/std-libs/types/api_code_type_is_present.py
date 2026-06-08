# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_code_type_is_present"
# subject = "types.CodeType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.CodeType: api_code_type_is_present (surface)."""
import types

assert hasattr(types, "CodeType")
print("api_code_type_is_present OK")
