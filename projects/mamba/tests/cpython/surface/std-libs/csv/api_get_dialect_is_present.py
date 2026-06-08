# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_get_dialect_is_present"
# subject = "csv.get_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.get_dialect: api_get_dialect_is_present (surface)."""
import csv

assert hasattr(csv, "get_dialect")
print("api_get_dialect_is_present OK")
