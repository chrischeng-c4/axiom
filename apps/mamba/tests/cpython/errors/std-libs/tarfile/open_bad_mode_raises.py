# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "open_bad_mode_raises"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: open_bad_mode_raises (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(), mode='X')
except (ValueError, tarfile.CompressionError):
    _raised = True
assert _raised, "open_bad_mode_raises: expected (ValueError, tarfile.CompressionError)"
print("open_bad_mode_raises OK")
