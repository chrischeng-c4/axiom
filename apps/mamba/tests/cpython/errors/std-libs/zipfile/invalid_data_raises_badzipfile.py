# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "invalid_data_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: invalid_data_raises_badzipfile (errors)."""
import zipfile
import io

_raised = False
try:
    zipfile.ZipFile(io.BytesIO(b'not a zip'))
except zipfile.BadZipFile:
    _raised = True
assert _raised, "invalid_data_raises_badzipfile: expected zipfile.BadZipFile"
print("invalid_data_raises_badzipfile OK")
