# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "surface"
# case = "register_dialect_is_callable"
# subject = "csv.register_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.register_dialect: register_dialect_is_callable (surface)."""
import csv

assert callable(csv.register_dialect)
print("register_dialect_is_callable OK")
