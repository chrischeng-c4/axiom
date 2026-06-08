# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_octdigits_is_present"
# subject = "string.octdigits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.octdigits: api_octdigits_is_present (surface)."""
import string

assert hasattr(string, "octdigits")
print("api_octdigits_is_present OK")
