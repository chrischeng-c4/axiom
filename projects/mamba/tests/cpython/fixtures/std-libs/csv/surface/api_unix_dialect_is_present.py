# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_unix_dialect_is_present"
# subject = "csv.unix_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.unix_dialect: api_unix_dialect_is_present (surface)."""
import csv

assert hasattr(csv, "unix_dialect")
print("api_unix_dialect_is_present OK")
