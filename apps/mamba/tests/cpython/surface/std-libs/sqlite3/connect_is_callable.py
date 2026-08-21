# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "connect_is_callable"
# subject = "sqlite3.connect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.connect: connect_is_callable (surface)."""
import sqlite3

assert callable(sqlite3.connect)
print("connect_is_callable OK")
