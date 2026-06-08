# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_ast_is_present"
# subject = "ast.AST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.AST: api_ast_is_present (surface)."""
import ast

assert hasattr(ast, "AST")
print("api_ast_is_present OK")
