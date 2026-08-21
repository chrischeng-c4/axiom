# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_tell_roundtrip_rewinds_lines"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: each tell()-recorded position rewinds to exactly the right line on seek (utf-8 multi-line round-trip)"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w+", encoding="utf-8") as f:
        p0 = f.tell()
        f.write("ÿ\n")
        p1 = f.tell()
        f.write("ÿ\n")
        p2 = f.tell()
        f.seek(0)
        assert f.tell() == p0, "start position"
        assert f.readline() == "ÿ\n", "line 1"
        assert f.tell() == p1, "after line 1"
        assert f.readline() == "ÿ\n", "line 2"
        assert f.tell() == p2, "after line 2"

print("text_tell_roundtrip_rewinds_lines OK")
