# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "errors"
# case = "writerows_propagates_file_error"
# subject = "csv.writer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""csv.writer: writerows_propagates_file_error (errors)."""
import csv
class _BrokenFile:
    def write(self, buf):
        raise OSError('boom')

_raised = False
try:
    csv.writer(_BrokenFile()).writerows([['a']])
except OSError:
    _raised = True
assert _raised, "writerows_propagates_file_error: expected OSError"
print("writerows_propagates_file_error OK")
