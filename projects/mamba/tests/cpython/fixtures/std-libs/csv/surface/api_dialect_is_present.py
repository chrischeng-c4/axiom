# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_dialect_is_present"
# subject = "csv.Dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.Dialect: api_dialect_is_present (surface)."""
import csv

assert hasattr(csv, "Dialect")
print("api_dialect_is_present OK")
