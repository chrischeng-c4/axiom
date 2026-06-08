# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_sqlite_limit_vdbe_op_is_present"
# subject = "sqlite3.SQLITE_LIMIT_VDBE_OP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.SQLITE_LIMIT_VDBE_OP: api_sqlite_limit_vdbe_op_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "SQLITE_LIMIT_VDBE_OP")
print("api_sqlite_limit_vdbe_op_is_present OK")
