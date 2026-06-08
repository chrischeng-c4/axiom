# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_nt_offset_is_present"
# subject = "tokenize.NT_OFFSET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.NT_OFFSET: api_nt_offset_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "NT_OFFSET")
print("api_nt_offset_is_present OK")
