# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_cell_type_is_present"
# subject = "types.CellType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.CellType: api_cell_type_is_present (surface)."""
import types

assert hasattr(types, "CellType")
print("api_cell_type_is_present OK")
