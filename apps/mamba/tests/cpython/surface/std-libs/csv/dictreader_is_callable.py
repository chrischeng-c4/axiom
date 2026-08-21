# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "dictreader_is_callable"
# subject = "csv.DictReader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.DictReader: dictreader_is_callable (surface)."""
import csv

assert callable(csv.DictReader)
print("dictreader_is_callable OK")
