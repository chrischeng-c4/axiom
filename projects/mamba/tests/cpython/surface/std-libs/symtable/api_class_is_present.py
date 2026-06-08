# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "surface"
# case = "api_class_is_present"
# subject = "symtable.Class"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""symtable.Class: api_class_is_present (surface)."""
import symtable

assert hasattr(symtable, "Class")
print("api_class_is_present OK")
