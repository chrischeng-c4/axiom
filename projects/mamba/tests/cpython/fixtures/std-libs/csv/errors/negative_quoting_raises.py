# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "negative_quoting_raises"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.reader: negative_quoting_raises (errors)."""
import csv

_raised = False
try:
    csv.reader([], quoting=-1)
except TypeError:
    _raised = True
assert _raised, "negative_quoting_raises: expected TypeError"
print("negative_quoting_raises OK")
