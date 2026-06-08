# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_capwords_is_present"
# subject = "string.capwords"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.capwords: api_capwords_is_present (surface)."""
import string

assert hasattr(string, "capwords")
print("api_capwords_is_present OK")
