# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "read_closed_stringio_raises"
# subject = "io.StringIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.StringIO: read_closed_stringio_raises (errors)."""
import io

_raised = False
try:
    _s = io.StringIO('text'); _s.close(); _s.read()
except ValueError:
    _raised = True
assert _raised, "read_closed_stringio_raises: expected ValueError"
print("read_closed_stringio_raises OK")
