# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_iter_child_nodes_is_present"
# subject = "ast.iter_child_nodes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.iter_child_nodes: api_iter_child_nodes_is_present (surface)."""
import ast

assert hasattr(ast, "iter_child_nodes")
print("api_iter_child_nodes_is_present OK")
