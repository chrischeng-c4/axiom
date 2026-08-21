# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "field_too_large_raises"
# subject = "csv.field_size_limit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.field_size_limit: field_too_large_raises (errors)."""
import csv

_raised = False
try:
    csv.field_size_limit(10) and list(csv.reader(["a" * 50 + ",b"]))
except csv.Error:
    _raised = True
assert _raised, "field_too_large_raises: expected csv.Error"
print("field_too_large_raises OK")
