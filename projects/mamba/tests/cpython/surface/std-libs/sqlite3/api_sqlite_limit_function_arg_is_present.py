# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_sqlite_limit_function_arg_is_present"
# subject = "sqlite3.SQLITE_LIMIT_FUNCTION_ARG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.SQLITE_LIMIT_FUNCTION_ARG: api_sqlite_limit_function_arg_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "SQLITE_LIMIT_FUNCTION_ARG")
print("api_sqlite_limit_function_arg_is_present OK")
