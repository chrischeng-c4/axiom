# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "surface"
# case = "api_aug_store_is_present"
# subject = "ast.AugStore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ast.AugStore: api_aug_store_is_present (surface)."""
import ast

assert hasattr(ast, "AugStore")
print("api_aug_store_is_present OK")
