# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_complete_statement_is_present"
# subject = "sqlite3.complete_statement"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.complete_statement: api_complete_statement_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "complete_statement")
print("api_complete_statement_is_present OK")
