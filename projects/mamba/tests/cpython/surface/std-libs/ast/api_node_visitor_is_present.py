# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_node_visitor_is_present"
# subject = "ast.NodeVisitor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.NodeVisitor: api_node_visitor_is_present (surface)."""
import ast

assert hasattr(ast, "NodeVisitor")
print("api_node_visitor_is_present OK")
