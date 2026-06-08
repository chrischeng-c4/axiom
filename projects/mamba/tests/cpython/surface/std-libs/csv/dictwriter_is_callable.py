# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "dictwriter_is_callable"
# subject = "csv.DictWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.DictWriter: dictwriter_is_callable (surface)."""
import csv

assert callable(csv.DictWriter)
print("dictwriter_is_callable OK")
