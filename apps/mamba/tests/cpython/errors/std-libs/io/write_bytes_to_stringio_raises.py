# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "write_bytes_to_stringio_raises"
# subject = "io.StringIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.StringIO: write_bytes_to_stringio_raises (errors)."""
import io

_raised = False
try:
    io.StringIO().write(b'bytes')
except TypeError:
    _raised = True
assert _raised, "write_bytes_to_stringio_raises: expected TypeError"
print("write_bytes_to_stringio_raises OK")
