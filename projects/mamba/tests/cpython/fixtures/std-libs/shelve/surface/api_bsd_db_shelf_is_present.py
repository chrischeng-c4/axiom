# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "surface"
# case = "api_bsd_db_shelf_is_present"
# subject = "shelve.BsdDbShelf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shelve.BsdDbShelf: api_bsd_db_shelf_is_present (surface)."""
import shelve

assert hasattr(shelve, "BsdDbShelf")
print("api_bsd_db_shelf_is_present OK")
