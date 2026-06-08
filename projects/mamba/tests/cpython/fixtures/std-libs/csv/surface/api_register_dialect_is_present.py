# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_register_dialect_is_present"
# subject = "csv.register_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.register_dialect: api_register_dialect_is_present (surface)."""
import csv

assert hasattr(csv, "register_dialect")
print("api_register_dialect_is_present OK")
