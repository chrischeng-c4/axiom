# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_seek_rewrite_preserves_single_bom"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: seeking past the BOM, then rewriting from the start, preserves exactly one leading BOM"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for charset in ("utf-8-sig", "utf-16", "utf-32"):
        with open(path, "w", encoding=charset) as f:
            f.write("aaa")
            pos = f.tell()
        with open(path, "r+", encoding=charset) as f:
            f.seek(pos)
            f.write("zzz")
            f.seek(0)
            f.write("bbb")
        with open(path, "rb") as f:
            assert f.read() == "bbbzzz".encode(charset), f"seek-rewrite {charset}"

print("text_seek_rewrite_preserves_single_bom OK")
