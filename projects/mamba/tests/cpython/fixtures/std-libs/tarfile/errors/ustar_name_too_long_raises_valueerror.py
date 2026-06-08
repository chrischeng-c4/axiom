# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "ustar_name_too_long_raises_valueerror"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: ustar_name_too_long_raises_valueerror (errors)."""
import tarfile

_raised = False
try:
    tarfile.TarInfo('0123456789' * 10 + '0').tobuf(tarfile.USTAR_FORMAT)
except ValueError:
    _raised = True
assert _raised, "ustar_name_too_long_raises_valueerror: expected ValueError"
print("ustar_name_too_long_raises_valueerror OK")
