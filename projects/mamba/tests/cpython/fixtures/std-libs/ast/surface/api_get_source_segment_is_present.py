# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_get_source_segment_is_present"
# subject = "ast.get_source_segment"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.get_source_segment: api_get_source_segment_is_present (surface)."""
import ast

assert hasattr(ast, "get_source_segment")
print("api_get_source_segment_is_present OK")
