# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "itn_too_big_ustar_raises_valueerror"
# subject = "tarfile.itn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn_too_big_ustar_raises_valueerror (errors)."""
import tarfile

_raised = False
try:
    tarfile.itn(2097152, 8, tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "itn_too_big_ustar_raises_valueerror: expected ValueError"
print("itn_too_big_ustar_raises_valueerror OK")
