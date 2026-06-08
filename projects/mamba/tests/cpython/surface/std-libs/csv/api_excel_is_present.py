# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_excel_is_present"
# subject = "csv.excel"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.excel: api_excel_is_present (surface)."""
import csv

assert hasattr(csv, "excel")
print("api_excel_is_present OK")
