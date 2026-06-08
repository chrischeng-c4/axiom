# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_dump_is_present"
# subject = "ast.dump"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.dump: api_dump_is_present (surface)."""
import ast

assert hasattr(ast, "dump")
print("api_dump_is_present OK")
