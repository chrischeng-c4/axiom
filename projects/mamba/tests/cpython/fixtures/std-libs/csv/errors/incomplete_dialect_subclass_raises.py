# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "incomplete_dialect_subclass_raises"
# subject = "csv.Dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.Dialect: incomplete_dialect_subclass_raises (errors)."""
import csv

_raised = False
try:
    type("_Incomplete", (csv.Dialect,), {"delimiter": "\\t"})()
except csv.Error:
    _raised = True
assert _raised, "incomplete_dialect_subclass_raises: expected csv.Error"
print("incomplete_dialect_subclass_raises OK")
