# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_compression_raises_notimplementederror"
# subject = "zipfile.ZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: bad_compression_raises_notimplementederror (errors)."""
import zipfile
import io

_raised = False
try:
    zipfile.ZipFile(io.BytesIO(), 'w', -1)
except NotImplementedError:
    _raised = True
assert _raised, "bad_compression_raises_notimplementederror: expected NotImplementedError"
print("bad_compression_raises_notimplementederror OK")
