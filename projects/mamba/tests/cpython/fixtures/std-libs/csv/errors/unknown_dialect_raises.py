# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "unknown_dialect_raises"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.reader: unknown_dialect_raises (errors)."""
import csv

_raised = False
try:
    csv.reader(["a,b"], dialect="no_such_dialect")
except csv.Error:
    _raised = True
assert _raised, "unknown_dialect_raises: expected csv.Error"
print("unknown_dialect_raises OK")
