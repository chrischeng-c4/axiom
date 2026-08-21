# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "whichdb_is_callable"
# subject = "dbm.whichdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm.whichdb: whichdb_is_callable (surface)."""
import dbm

assert callable(dbm.whichdb)
print("whichdb_is_callable OK")
