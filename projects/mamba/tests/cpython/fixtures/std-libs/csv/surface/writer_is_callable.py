# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "writer_is_callable"
# subject = "csv.writer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.writer: writer_is_callable (surface)."""
import csv

assert callable(csv.writer)
print("writer_is_callable OK")
