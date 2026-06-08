# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "row_class_is_callable"
# subject = "sqlite3.Row"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Row: row_class_is_callable (surface)."""
import sqlite3

assert callable(sqlite3.Row)
print("row_class_is_callable OK")
