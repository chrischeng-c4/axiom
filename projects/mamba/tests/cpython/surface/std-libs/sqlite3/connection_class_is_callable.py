# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "connection_class_is_callable"
# subject = "sqlite3.Connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: connection_class_is_callable (surface)."""
import sqlite3

assert callable(sqlite3.Connection)
print("connection_class_is_callable OK")
