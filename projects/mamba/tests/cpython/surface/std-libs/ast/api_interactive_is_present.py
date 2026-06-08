# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_interactive_is_present"
# subject = "ast.Interactive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Interactive: api_interactive_is_present (surface)."""
import ast

assert hasattr(ast, "Interactive")
print("api_interactive_is_present OK")
