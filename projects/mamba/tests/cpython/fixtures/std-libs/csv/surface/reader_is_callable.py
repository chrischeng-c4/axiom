# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "reader_is_callable"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.reader: reader_is_callable (surface)."""
import csv

assert callable(csv.reader)
print("reader_is_callable OK")
