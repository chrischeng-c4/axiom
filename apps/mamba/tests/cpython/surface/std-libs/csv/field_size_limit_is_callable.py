# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "field_size_limit_is_callable"
# subject = "csv.field_size_limit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.field_size_limit: field_size_limit_is_callable (surface)."""
import csv

assert callable(csv.field_size_limit)
print("field_size_limit_is_callable OK")
