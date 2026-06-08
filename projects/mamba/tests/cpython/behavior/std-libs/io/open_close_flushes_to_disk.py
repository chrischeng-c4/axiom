# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_close_flushes_to_disk"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: close() flushes buffered writes so a fresh read sees the full content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"flushed")
    with open(path, "rb") as f:
        assert f.read() == b"flushed", "close did not flush"

print("open_close_flushes_to_disk OK")
