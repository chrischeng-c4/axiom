# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "compresslevel_plain_mode_raises_typeerror"
# subject = "tarfile.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tarfile.open: compresslevel_plain_mode_raises_typeerror (errors)."""
import tarfile
import io

_raised = False
try:
    tarfile.open(fileobj=io.BytesIO(), mode='w:', compresslevel=5)
except TypeError:
    _raised = True
assert _raised, "compresslevel_plain_mode_raises_typeerror: expected TypeError"
print("compresslevel_plain_mode_raises_typeerror OK")
