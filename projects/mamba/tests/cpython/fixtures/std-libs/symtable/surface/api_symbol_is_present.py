# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "surface"
# case = "api_symbol_is_present"
# subject = "symtable.Symbol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""symtable.Symbol: api_symbol_is_present (surface)."""
import symtable

assert hasattr(symtable, "Symbol")
print("api_symbol_is_present OK")
