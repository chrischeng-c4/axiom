# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "open_non_tar_raises_readerror"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.open: open_non_tar_raises_readerror (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(b'not a tar file'))
except tarfile.ReadError:
    _raised = True
assert _raised, "open_non_tar_raises_readerror: expected tarfile.ReadError"
print("open_non_tar_raises_readerror OK")
