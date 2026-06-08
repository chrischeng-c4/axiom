# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "unknown_kwarg_raises"
# subject = "csv.reader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.reader: unknown_kwarg_raises (errors)."""
import csv

_raised = False
try:
    csv.reader([], bad_attr=0)
except TypeError:
    _raised = True
assert _raised, "unknown_kwarg_raises: expected TypeError"
print("unknown_kwarg_raises OK")
