# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "compresslevel_bz2_zero_raises_valueerror"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: compresslevel_bz2_zero_raises_valueerror (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(), mode='w:bz2', compresslevel=0)
except ValueError:
    _raised = True
assert _raised, "compresslevel_bz2_zero_raises_valueerror: expected ValueError"
print("compresslevel_bz2_zero_raises_valueerror OK")
