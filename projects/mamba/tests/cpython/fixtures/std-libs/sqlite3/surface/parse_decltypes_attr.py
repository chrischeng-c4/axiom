# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "parse_decltypes_attr"
# subject = "sqlite3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3: parse_decltypes_attr (surface)."""
import sqlite3

assert hasattr(sqlite3, "PARSE_DECLTYPES")
print("parse_decltypes_attr OK")
