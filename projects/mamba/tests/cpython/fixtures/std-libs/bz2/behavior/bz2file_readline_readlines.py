# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_readline_readlines"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File supports readline then readlines, splitting a multi-line payload correctly"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"alpha\nbeta\ngamma\n")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    line = f.readline()
    assert line == b"alpha\n", f"readline = {line!r}"
    rest = f.readlines()
    assert rest == [b"beta\n", b"gamma\n"], f"readlines = {rest!r}"
print("bz2file_readline_readlines OK")
