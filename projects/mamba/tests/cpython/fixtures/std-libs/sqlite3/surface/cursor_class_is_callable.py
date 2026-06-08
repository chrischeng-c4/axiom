# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "cursor_class_is_callable"
# subject = "sqlite3.Cursor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Cursor: cursor_class_is_callable (surface)."""
import sqlite3

assert callable(sqlite3.Cursor)
print("cursor_class_is_callable OK")
