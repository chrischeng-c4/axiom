# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "missing_table_raises"
# subject = "sqlite3.Connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: missing_table_raises (errors)."""
import sqlite3

_raised = False
try:
    sqlite3.connect(":memory:").execute("SELECT * FROM nope_table")
except sqlite3.OperationalError:
    _raised = True
assert _raised, "missing_table_raises: expected sqlite3.OperationalError"
print("missing_table_raises OK")
