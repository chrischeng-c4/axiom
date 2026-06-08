# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "surface"
# case = "api_dbfilename_shelf_is_present"
# subject = "shelve.DbfilenameShelf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shelve.DbfilenameShelf: api_dbfilename_shelf_is_present (surface)."""
import shelve

assert hasattr(shelve, "DbfilenameShelf")
print("api_dbfilename_shelf_is_present OK")
