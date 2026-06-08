# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "surface"
# case = "api_function_is_present"
# subject = "symtable.Function"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""symtable.Function: api_function_is_present (surface)."""
import symtable

assert hasattr(symtable, "Function")
print("api_function_is_present OK")
