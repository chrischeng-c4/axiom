# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "seek_bad_whence_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: seek_bad_whence_raises (errors)."""
import io

_raised = False
try:
    io.BytesIO(b'data').seek(0, 99)
except ValueError:
    _raised = True
assert _raised, "seek_bad_whence_raises: expected ValueError"
print("seek_bad_whence_raises OK")
