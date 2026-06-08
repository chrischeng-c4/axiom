# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_class_def_is_present"
# subject = "ast.ClassDef"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.ClassDef: api_class_def_is_present (surface)."""
import ast

assert hasattr(ast, "ClassDef")
print("api_class_def_is_present OK")
