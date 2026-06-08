# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_hexdigits_is_present"
# subject = "string.hexdigits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.hexdigits: api_hexdigits_is_present (surface)."""
import string

assert hasattr(string, "hexdigits")
print("api_hexdigits_is_present OK")
