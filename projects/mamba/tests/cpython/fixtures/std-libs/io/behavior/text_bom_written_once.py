# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_bom_written_once"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: BOM-bearing encodings (utf-8-sig/utf-16/utf-32) write the BOM exactly once; appending does not re-emit it"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for charset in ("utf-8-sig", "utf-16", "utf-32"):
        with open(path, "w", encoding=charset) as f:
            f.write("aaa")
        with open(path, "rb") as f:
            assert f.read() == "aaa".encode(charset), f"BOM write {charset}"
        with open(path, "a", encoding=charset) as f:
            f.write("xxx")
        with open(path, "rb") as f:
            assert f.read() == "aaaxxx".encode(charset), f"append no re-BOM {charset}"

print("text_bom_written_once OK")
