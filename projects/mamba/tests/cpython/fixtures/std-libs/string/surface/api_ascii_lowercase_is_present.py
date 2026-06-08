# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_ascii_lowercase_is_present"
# subject = "string.ascii_lowercase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.ascii_lowercase: api_ascii_lowercase_is_present (surface)."""
import string

assert hasattr(string, "ascii_lowercase")
print("api_ascii_lowercase_is_present OK")
