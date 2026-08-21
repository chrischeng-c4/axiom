# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "errors"
# case = "connect_directory_raises"
# subject = "sqlite3.connect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.connect: connect_directory_raises (errors)."""
import sqlite3

_raised = False
try:
    sqlite3.connect("/")
except sqlite3.OperationalError:
    _raised = True
assert _raised, "connect_directory_raises: expected sqlite3.OperationalError"
print("connect_directory_raises OK")
