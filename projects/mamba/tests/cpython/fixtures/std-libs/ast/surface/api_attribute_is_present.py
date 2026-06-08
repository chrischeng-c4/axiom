# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_attribute_is_present"
# subject = "ast.Attribute"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.Attribute: api_attribute_is_present (surface)."""
import ast

assert hasattr(ast, "Attribute")
print("api_attribute_is_present OK")
