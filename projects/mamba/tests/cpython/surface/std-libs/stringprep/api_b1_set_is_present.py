# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "surface"
# case = "api_b1_set_is_present"
# subject = "stringprep.b1_set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stringprep.b1_set: api_b1_set_is_present (surface)."""
import stringprep

assert hasattr(stringprep, "b1_set")
print("api_b1_set_is_present OK")
