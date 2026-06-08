# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_whitespace_is_present"
# subject = "string.whitespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.whitespace: api_whitespace_is_present (surface)."""
import string

assert hasattr(string, "whitespace")
print("api_whitespace_is_present OK")
