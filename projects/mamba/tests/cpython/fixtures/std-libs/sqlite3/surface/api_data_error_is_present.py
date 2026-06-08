# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_data_error_is_present"
# subject = "sqlite3.DataError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.DataError: api_data_error_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "DataError")
print("api_data_error_is_present OK")
