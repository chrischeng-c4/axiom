# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_constructor_mode_raises_valueerror"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: bad_constructor_mode_raises_valueerror (errors)."""
import zipfile
import io

_raised = False
try:
    zipfile.ZipFile(io.BytesIO(), 'q')
except ValueError:
    _raised = True
assert _raised, "bad_constructor_mode_raises_valueerror: expected ValueError"
print("bad_constructor_mode_raises_valueerror OK")
