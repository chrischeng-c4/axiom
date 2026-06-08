# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "write_str_to_bytesio_raises"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BytesIO: write_str_to_bytesio_raises (errors)."""
import io

_raised = False
try:
    io.BytesIO().write('str')
except TypeError:
    _raised = True
assert _raised, "write_str_to_bytesio_raises: expected TypeError"
print("write_str_to_bytesio_raises OK")
