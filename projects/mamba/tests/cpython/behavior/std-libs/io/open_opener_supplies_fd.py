# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_opener_supplies_fd"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: an opener callback supplies the underlying fd and the filename argument is ignored"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"abcdef")
    fd = os.open(path, os.O_RDONLY)
    with open("ignored", "rb", opener=lambda p, flags: fd) as f:
        assert f.read()[:3] == b"abc", "opener-supplied fd read"

print("open_opener_supplies_fd OK")
