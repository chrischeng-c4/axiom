# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_div_is_present"
# subject = "ast.Div"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Div: api_div_is_present (surface)."""
import ast

assert hasattr(ast, "Div")
print("api_div_is_present OK")
