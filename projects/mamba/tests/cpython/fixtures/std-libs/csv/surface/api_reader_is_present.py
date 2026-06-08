# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "api_reader_is_present"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""csv.reader: api_reader_is_present (surface)."""
import csv

assert hasattr(csv, "reader")
print("api_reader_is_present OK")
