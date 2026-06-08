# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "sqlite3.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.Error: api_error_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "Error")
print("api_error_is_present OK")
