# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "surface"
# case = "api_c22_specials_is_present"
# subject = "stringprep.c22_specials"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stringprep.c22_specials: api_c22_specials_is_present (surface)."""
import stringprep

assert hasattr(stringprep, "c22_specials")
print("api_c22_specials_is_present OK")
