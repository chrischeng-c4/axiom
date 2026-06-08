# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "surface"
# case = "api_typecodes_is_present"
# subject = "array.typecodes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""array.typecodes: api_typecodes_is_present (surface)."""
import array

assert hasattr(array, "typecodes")
print("api_typecodes_is_present OK")
