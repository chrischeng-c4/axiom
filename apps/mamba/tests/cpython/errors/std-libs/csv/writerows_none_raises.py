# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "writerows_none_raises"
# subject = "csv.writer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.writer: writerows_none_raises (errors)."""
import csv

_raised = False
try:
    csv.writer([]).writerows(None)
except TypeError:
    _raised = True
assert _raised, "writerows_none_raises: expected TypeError"
print("writerows_none_raises OK")
