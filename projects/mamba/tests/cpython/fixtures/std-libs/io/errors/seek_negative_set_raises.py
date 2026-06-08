# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "seek_negative_set_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: seek_negative_set_raises (errors)."""
import io

_raised = False
try:
    io.BytesIO(b'data').seek(-1, 0)
except ValueError:
    _raised = True
assert _raised, "seek_negative_set_raises: expected ValueError"
print("seek_negative_set_raises OK")
