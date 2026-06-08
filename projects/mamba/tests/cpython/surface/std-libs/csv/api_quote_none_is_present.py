# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_quote_none_is_present"
# subject = "csv.QUOTE_NONE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.QUOTE_NONE: api_quote_none_is_present (surface)."""
import csv

assert hasattr(csv, "QUOTE_NONE")
print("api_quote_none_is_present OK")
