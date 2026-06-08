# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_sqlite_ok_load_permanently_is_present"
# subject = "sqlite3.SQLITE_OK_LOAD_PERMANENTLY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.SQLITE_OK_LOAD_PERMANENTLY: api_sqlite_ok_load_permanently_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "SQLITE_OK_LOAD_PERMANENTLY")
print("api_sqlite_ok_load_permanently_is_present OK")
