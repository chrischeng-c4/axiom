# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "list_dialects_is_callable"
# subject = "csv.list_dialects"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.list_dialects: list_dialects_is_callable (surface)."""
import csv

assert callable(csv.list_dialects)
print("list_dialects_is_callable OK")
