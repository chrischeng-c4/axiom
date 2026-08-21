# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "quote_all_is_int"
# subject = "csv.QUOTE_ALL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.QUOTE_ALL: quote_all_is_int (surface)."""
import csv

assert type(csv.QUOTE_ALL).__name__ == "int"
print("quote_all_is_int OK")
