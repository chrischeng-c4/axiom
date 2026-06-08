# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "error_is_present"
# subject = "csv.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.Error: error_is_present (surface)."""
import csv

assert callable(csv.Error)
print("error_is_present OK")
