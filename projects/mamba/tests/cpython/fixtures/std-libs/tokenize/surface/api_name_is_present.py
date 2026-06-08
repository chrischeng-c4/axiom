# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_name_is_present"
# subject = "tokenize.NAME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.NAME: api_name_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "NAME")
print("api_name_is_present OK")
