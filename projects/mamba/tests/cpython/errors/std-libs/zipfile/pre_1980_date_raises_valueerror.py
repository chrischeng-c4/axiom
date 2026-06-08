# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "pre_1980_date_raises_valueerror"
# subject = "zipfile.ZipInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipInfo: pre_1980_date_raises_valueerror (errors)."""
import zipfile

_raised = False
try:
    zipfile.ZipInfo('old', (1979, 1, 1, 0, 0, 0))
except ValueError:
    _raised = True
assert _raised, "pre_1980_date_raises_valueerror: expected ValueError"
print("pre_1980_date_raises_valueerror OK")
