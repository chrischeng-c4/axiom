# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getcwdb_is_present"
# subject = "os.getcwdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getcwdb: api_getcwdb_is_present (surface)."""
import os

assert hasattr(os, "getcwdb")
print("api_getcwdb_is_present OK")
