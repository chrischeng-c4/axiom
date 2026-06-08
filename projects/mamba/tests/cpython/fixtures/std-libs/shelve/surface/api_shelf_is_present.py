# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "surface"
# case = "api_shelf_is_present"
# subject = "shelve.Shelf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shelve.Shelf: api_shelf_is_present (surface)."""
import shelve

assert hasattr(shelve, "Shelf")
print("api_shelf_is_present OK")
