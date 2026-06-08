# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "bdb_class_is_callable"
# subject = "bdb.Bdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb: bdb_class_is_callable (surface)."""
import bdb

assert callable(bdb.Bdb)
print("bdb_class_is_callable OK")
