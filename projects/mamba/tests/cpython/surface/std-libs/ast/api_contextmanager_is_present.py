# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_contextmanager_is_present"
# subject = "ast.contextmanager"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.contextmanager: api_contextmanager_is_present (surface)."""
import ast

assert hasattr(ast, "contextmanager")
print("api_contextmanager_is_present OK")
