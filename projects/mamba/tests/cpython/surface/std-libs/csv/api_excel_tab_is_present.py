# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_excel_tab_is_present"
# subject = "csv.excel_tab"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.excel_tab: api_excel_tab_is_present (surface)."""
import csv

assert hasattr(csv, "excel_tab")
print("api_excel_tab_is_present OK")
