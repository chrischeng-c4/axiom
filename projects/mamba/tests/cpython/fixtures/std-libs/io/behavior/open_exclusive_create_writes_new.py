# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_exclusive_create_writes_new"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: exclusive-create mode 'xb' writes a brand-new file and round-trips its content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    new = os.path.join(d, "new.bin")
    with open(new, "xb") as f:
        f.write(b"spam")
    with open(new, "rb") as f:
        assert f.read() == b"spam", "xb create round-trip"

print("open_exclusive_create_writes_new OK")
