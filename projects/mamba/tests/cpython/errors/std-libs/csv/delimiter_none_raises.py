# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "delimiter_none_raises"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.reader: delimiter_none_raises (errors)."""
import csv

_raised = False
try:
    csv.reader([], delimiter=None)
except TypeError:
    _raised = True
assert _raised, "delimiter_none_raises: expected TypeError"
print("delimiter_none_raises OK")
