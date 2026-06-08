# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_indent_is_present"
# subject = "tokenize.INDENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.INDENT: api_indent_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "INDENT")
print("api_indent_is_present OK")
