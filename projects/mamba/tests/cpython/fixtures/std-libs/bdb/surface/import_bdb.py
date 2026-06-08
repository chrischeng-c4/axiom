# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "import_bdb"
# subject = "bdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb: import_bdb (surface)."""
import bdb

assert hasattr(bdb, "Bdb")
print("import_bdb OK")
