# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_quote_all_is_present"
# subject = "csv.QUOTE_ALL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.QUOTE_ALL: api_quote_all_is_present (surface)."""
import csv

assert hasattr(csv, "QUOTE_ALL")
print("api_quote_all_is_present OK")
