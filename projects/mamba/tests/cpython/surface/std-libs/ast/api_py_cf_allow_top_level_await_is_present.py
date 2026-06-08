# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_py_cf_allow_top_level_await_is_present"
# subject = "ast.PyCF_ALLOW_TOP_LEVEL_AWAIT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.PyCF_ALLOW_TOP_LEVEL_AWAIT: api_py_cf_allow_top_level_await_is_present (surface)."""
import ast

assert hasattr(ast, "PyCF_ALLOW_TOP_LEVEL_AWAIT")
print("api_py_cf_allow_top_level_await_is_present OK")
