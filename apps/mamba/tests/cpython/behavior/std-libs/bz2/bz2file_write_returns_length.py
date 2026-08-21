# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_write_returns_length"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File.write returns the byte length (incl. buffer-protocol objects) and tell reflects bytes written"""
import array
import bz2
import io

q = array.array("Q", [1, 2, 3, 4, 5])
length = len(q) * q.itemsize
with bz2.BZ2File(io.BytesIO(), "w") as f:
    assert f.write(q) == length, "write returns buffer byte length"
    assert f.tell() == length, f"tell = {f.tell()}"
print("bz2file_write_returns_length OK")
