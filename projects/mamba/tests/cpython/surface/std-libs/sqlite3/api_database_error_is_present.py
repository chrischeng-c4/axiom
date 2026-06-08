# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_database_error_is_present"
# subject = "sqlite3.DatabaseError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.DatabaseError: api_database_error_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "DatabaseError")
print("api_database_error_is_present OK")
