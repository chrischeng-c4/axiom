# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_punctuation_is_present"
# subject = "string.punctuation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.punctuation: api_punctuation_is_present (surface)."""
import string

assert hasattr(string, "punctuation")
print("api_punctuation_is_present OK")
