# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "bad_sql_raises"
# subject = "sqlite3.Connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Connection: bad_sql_raises (errors)."""
import sqlite3

_raised = False
try:
    sqlite3.connect(":memory:").execute("NOT VALID SQL")
except sqlite3.OperationalError:
    _raised = True
assert _raised, "bad_sql_raises: expected sqlite3.OperationalError"
print("bad_sql_raises OK")
