# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "itn_negative_ustar_raises_valueerror"
# subject = "tarfile.itn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn_negative_ustar_raises_valueerror (errors)."""
import tarfile

_raised = False
try:
    tarfile.itn(-1, 8, tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "itn_negative_ustar_raises_valueerror: expected ValueError"
print("itn_negative_ustar_raises_valueerror OK")
