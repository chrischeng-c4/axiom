# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_sqlite_detach_is_present"
# subject = "sqlite3.SQLITE_DETACH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.SQLITE_DETACH: api_sqlite_detach_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "SQLITE_DETACH")
print("api_sqlite_detach_is_present OK")
