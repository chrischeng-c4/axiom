# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_quote_strings_is_present"
# subject = "csv.QUOTE_STRINGS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.QUOTE_STRINGS: api_quote_strings_is_present (surface)."""
import csv

assert hasattr(csv, "QUOTE_STRINGS")
print("api_quote_strings_is_present OK")
