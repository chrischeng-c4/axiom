# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_op_is_present"
# subject = "tokenize.OP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.OP: api_op_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "OP")
print("api_op_is_present OK")
