# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "out_of_range_quoting_raises"
# subject = "csv.writer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.writer: out_of_range_quoting_raises (errors)."""
import csv

_raised = False
try:
    csv.writer([], quoting=999)
except TypeError:
    _raised = True
assert _raised, "out_of_range_quoting_raises: expected TypeError"
print("out_of_range_quoting_raises OK")
