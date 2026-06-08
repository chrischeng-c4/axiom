# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "surface"
# case = "api_c8_set_is_present"
# subject = "stringprep.c8_set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stringprep.c8_set: api_c8_set_is_present (surface)."""
import stringprep

assert hasattr(stringprep, "c8_set")
print("api_c8_set_is_present OK")
