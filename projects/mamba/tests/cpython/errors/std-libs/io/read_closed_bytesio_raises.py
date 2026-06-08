# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "read_closed_bytesio_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: read_closed_bytesio_raises (errors)."""
import io

_raised = False
try:
    _b = io.BytesIO(b'hello'); _b.close(); _b.read()
except ValueError:
    _raised = True
assert _raised, "read_closed_bytesio_raises: expected ValueError"
print("read_closed_bytesio_raises OK")
