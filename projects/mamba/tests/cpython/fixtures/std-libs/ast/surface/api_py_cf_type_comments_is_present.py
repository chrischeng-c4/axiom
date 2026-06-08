# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_py_cf_type_comments_is_present"
# subject = "ast.PyCF_TYPE_COMMENTS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.PyCF_TYPE_COMMENTS: api_py_cf_type_comments_is_present (surface)."""
import ast

assert hasattr(ast, "PyCF_TYPE_COMMENTS")
print("api_py_cf_type_comments_is_present OK")
