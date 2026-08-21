# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "import_sqlite3"
# subject = "sqlite3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3: import_sqlite3 (surface)."""
import sqlite3

assert hasattr(sqlite3, "connect")
print("import_sqlite3 OK")
