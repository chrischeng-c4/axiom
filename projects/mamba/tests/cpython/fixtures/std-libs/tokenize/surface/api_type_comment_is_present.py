# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_type_comment_is_present"
# subject = "tokenize.TYPE_COMMENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.TYPE_COMMENT: api_type_comment_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "TYPE_COMMENT")
print("api_type_comment_is_present OK")
