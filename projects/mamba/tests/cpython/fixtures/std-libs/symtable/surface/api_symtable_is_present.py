# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "surface"
# case = "api_symtable_is_present"
# subject = "symtable.symtable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""symtable.symtable: api_symtable_is_present (surface)."""
import symtable

assert hasattr(symtable, "symtable")
print("api_symtable_is_present OK")
