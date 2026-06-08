# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "dialect_is_present"
# subject = "csv.Dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.Dialect: dialect_is_present (surface)."""
import csv

assert callable(csv.Dialect)
print("dialect_is_present OK")
