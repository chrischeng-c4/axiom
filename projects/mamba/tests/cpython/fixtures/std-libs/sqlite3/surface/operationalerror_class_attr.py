# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "operationalerror_class_attr"
# subject = "sqlite3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3: operationalerror_class_attr (surface)."""
import sqlite3

assert hasattr(sqlite3, "OperationalError")
print("operationalerror_class_attr OK")
