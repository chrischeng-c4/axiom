# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "get_unknown_dialect_raises"
# subject = "csv.get_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.get_dialect: get_unknown_dialect_raises (errors)."""
import csv

_raised = False
try:
    csv.get_dialect("nonesuch")
except csv.Error:
    _raised = True
assert _raised, "get_unknown_dialect_raises: expected csv.Error"
print("get_unknown_dialect_raises OK")
