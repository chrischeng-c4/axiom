# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "quote_minimal_is_zero"
# subject = "csv.QUOTE_MINIMAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.QUOTE_MINIMAL: quote_minimal_is_zero (surface)."""
import csv

assert hasattr(csv.QUOTE_MINIMAL, "real")
print("quote_minimal_is_zero OK")
