# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_quote_nonnumeric_is_present"
# subject = "csv.QUOTE_NONNUMERIC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.QUOTE_NONNUMERIC: api_quote_nonnumeric_is_present (surface)."""
import csv

assert hasattr(csv, "QUOTE_NONNUMERIC")
print("api_quote_nonnumeric_is_present OK")
