# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_node_transformer_is_present"
# subject = "ast.NodeTransformer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.NodeTransformer: api_node_transformer_is_present (surface)."""
import ast

assert hasattr(ast, "NodeTransformer")
print("api_node_transformer_is_present OK")
