# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_close_is_idempotent"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: close() is idempotent (repeated calls are safe) but flush() on a closed file raises ValueError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    f = open(path, "wb", buffering=0)
    f.close()
    f.close()
    f.close()
    flush_raised = False
    try:
        f.flush()
    except ValueError:
        flush_raised = True
    assert flush_raised, "flush on closed file did not raise ValueError"

print("open_close_is_idempotent OK")
