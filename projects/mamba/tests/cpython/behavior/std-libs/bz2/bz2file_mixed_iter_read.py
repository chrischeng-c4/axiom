# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_mixed_iter_read"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File iteration (next) and read share one position so readline, next, and read consume in order"""
import bz2
import io

src = io.BytesIO()
with bz2.BZ2File(src, "wb") as f:
    f.write(b"alpha\nbeta\ngamma\n")
src.seek(0)
with bz2.BZ2File(src, "rb") as f:
    f.readline()
    assert next(f) == b"beta\n", "next after readline"
    assert f.read() == b"gamma\n", "read tail"
print("bz2file_mixed_iter_read OK")
