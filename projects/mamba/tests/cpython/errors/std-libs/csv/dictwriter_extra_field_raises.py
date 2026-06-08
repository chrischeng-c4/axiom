# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "dictwriter_extra_field_raises"
# subject = "csv.DictWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.DictWriter: dictwriter_extra_field_raises (errors)."""
import csv

_raised = False
try:
    csv.DictWriter(__import__("io").StringIO(), fieldnames=["a", "b"], extrasaction="raise").writerow({"a": 1, "b": 2, "extra": 3})
except ValueError:
    _raised = True
assert _raised, "dictwriter_extra_field_raises: expected ValueError"
print("dictwriter_extra_field_raises OK")
