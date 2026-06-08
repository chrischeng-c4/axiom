# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_dbapi2_is_present"
# subject = "sqlite3.dbapi2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.dbapi2: api_dbapi2_is_present (surface)."""
import sqlite3.dbapi2

assert hasattr(sqlite3, "dbapi2")
print("api_dbapi2_is_present OK")
