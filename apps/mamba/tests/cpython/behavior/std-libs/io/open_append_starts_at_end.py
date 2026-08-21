# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_append_starts_at_end"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: append mode ('ab'/'a') positions the new handle at the end of existing content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"xxx")
    with open(path, "ab", buffering=0) as f:
        assert f.tell() == 3, f"append binary tell = {f.tell()!r}"
    with open(path, "a", encoding="utf-8") as f:
        assert f.tell() > 0, f"append text tell = {f.tell()!r}"

print("open_append_starts_at_end OK")
