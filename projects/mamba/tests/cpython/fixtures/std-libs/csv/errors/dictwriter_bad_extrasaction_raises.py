# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "dictwriter_bad_extrasaction_raises"
# subject = "csv.DictWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.DictWriter: dictwriter_bad_extrasaction_raises (errors)."""
import csv

_raised = False
try:
    csv.DictWriter(__import__("io").StringIO(), ["f1", "f2"], extrasaction="raised")
except ValueError:
    _raised = True
assert _raised, "dictwriter_bad_extrasaction_raises: expected ValueError"
print("dictwriter_bad_extrasaction_raises OK")
