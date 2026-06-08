# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_py_cf_only_ast_is_present"
# subject = "ast.PyCF_ONLY_AST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.PyCF_ONLY_AST: api_py_cf_only_ast_is_present (surface)."""
import ast

assert hasattr(ast, "PyCF_ONLY_AST")
print("api_py_cf_only_ast_is_present OK")
