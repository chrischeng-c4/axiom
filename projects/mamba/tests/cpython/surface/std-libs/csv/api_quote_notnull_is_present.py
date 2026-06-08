# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_quote_notnull_is_present"
# subject = "csv.QUOTE_NOTNULL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.QUOTE_NOTNULL: api_quote_notnull_is_present (surface)."""
import csv

assert hasattr(csv, "QUOTE_NOTNULL")
print("api_quote_notnull_is_present OK")
