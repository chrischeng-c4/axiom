# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_comment_is_present"
# subject = "tokenize.COMMENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.COMMENT: api_comment_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "COMMENT")
print("api_comment_is_present OK")
