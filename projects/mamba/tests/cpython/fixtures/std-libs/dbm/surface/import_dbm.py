# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "import_dbm"
# subject = "dbm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dbm: import_dbm (surface)."""
import dbm

assert hasattr(dbm, "open")
print("import_dbm OK")
