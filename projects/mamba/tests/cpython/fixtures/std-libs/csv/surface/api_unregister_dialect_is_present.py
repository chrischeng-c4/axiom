# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_unregister_dialect_is_present"
# subject = "csv.unregister_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.unregister_dialect: api_unregister_dialect_is_present (surface)."""
import csv

assert hasattr(csv, "unregister_dialect")
print("api_unregister_dialect_is_present OK")
