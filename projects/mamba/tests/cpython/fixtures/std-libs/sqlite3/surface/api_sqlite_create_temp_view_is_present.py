# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_sqlite_create_temp_view_is_present"
# subject = "sqlite3.SQLITE_CREATE_TEMP_VIEW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.SQLITE_CREATE_TEMP_VIEW: api_sqlite_create_temp_view_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "SQLITE_CREATE_TEMP_VIEW")
print("api_sqlite_create_temp_view_is_present OK")
