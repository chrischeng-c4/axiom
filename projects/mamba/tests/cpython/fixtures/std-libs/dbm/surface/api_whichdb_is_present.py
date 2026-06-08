# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "api_whichdb_is_present"
# subject = "dbm.whichdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dbm.whichdb: api_whichdb_is_present (surface)."""
import dbm

assert hasattr(dbm, "whichdb")
print("api_whichdb_is_present OK")
