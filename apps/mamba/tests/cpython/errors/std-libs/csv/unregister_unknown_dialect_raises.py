# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "unregister_unknown_dialect_raises"
# subject = "csv.unregister_dialect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.unregister_dialect: unregister_unknown_dialect_raises (errors)."""
import csv

_raised = False
try:
    csv.unregister_dialect("nonesuch")
except csv.Error:
    _raised = True
assert _raised, "unregister_unknown_dialect_raises: expected csv.Error"
print("unregister_unknown_dialect_raises OK")
