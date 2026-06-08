# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_excepthandler_is_present"
# subject = "ast.excepthandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.excepthandler: api_excepthandler_is_present (surface)."""
import ast

assert hasattr(ast, "excepthandler")
print("api_excepthandler_is_present OK")
