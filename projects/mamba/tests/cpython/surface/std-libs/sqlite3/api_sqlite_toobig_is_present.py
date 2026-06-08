# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_sqlite_toobig_is_present"
# subject = "sqlite3.SQLITE_TOOBIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.SQLITE_TOOBIG: api_sqlite_toobig_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "SQLITE_TOOBIG")
print("api_sqlite_toobig_is_present OK")
