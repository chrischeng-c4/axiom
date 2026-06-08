# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "getcwdb_is_callable"
# subject = "os.getcwdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getcwdb: getcwdb_is_callable (surface)."""
import os

assert callable(os.getcwdb)
print("getcwdb_is_callable OK")
