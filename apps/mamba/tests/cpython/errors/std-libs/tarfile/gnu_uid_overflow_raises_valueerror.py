# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "gnu_uid_overflow_raises_valueerror"
# subject = "tarfile.TarInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarInfo: gnu_uid_overflow_raises_valueerror (errors)."""
import tarfile
_ti = tarfile.TarInfo('name')
_ti.uid = 72057594037927936

_raised = False
try:
    _ti.tobuf(tarfile.GNU_FORMAT)
except ValueError:
    _raised = True
assert _raised, "gnu_uid_overflow_raises_valueerror: expected ValueError"
print("gnu_uid_overflow_raises_valueerror OK")
