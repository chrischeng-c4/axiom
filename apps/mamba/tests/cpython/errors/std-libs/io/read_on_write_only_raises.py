# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "read_on_write_only_raises"
# subject = "io.BufferedWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BufferedWriter: read_on_write_only_raises (errors)."""
import io

_raised = False
try:
    io.BufferedWriter(io.BytesIO()).read(5)
except io.UnsupportedOperation:
    _raised = True
assert _raised, "read_on_write_only_raises: expected io.UnsupportedOperation"
print("read_on_write_only_raises OK")
