# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "api_array_type_is_present"
# subject = "array.ArrayType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""array.ArrayType: api_array_type_is_present (surface)."""
import array

assert hasattr(array, "ArrayType")
print("api_array_type_is_present OK")
