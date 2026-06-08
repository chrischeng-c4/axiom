# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_ellipsis_is_present"
# subject = "tokenize.ELLIPSIS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.ELLIPSIS: api_ellipsis_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "ELLIPSIS")
print("api_ellipsis_is_present OK")
