# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_seek_modes"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: BZ2File seek works absolute, relative (whence=1) and from-end (whence=2) on a seekable read stream"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"0123456789abcdef")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    assert f.seekable() is True, "seekable in read mode"
    f.seek(8)
    assert f.read(4) == b"89ab", "seek forward absolute"
    # Now at position 12; rewind 8 bytes relative to land back at 4.
    f.seek(-8, 1)
    assert f.read(4) == b"4567", "seek backward relative"
    f.seek(-4, 2)
    assert f.read() == b"cdef", "seek from end"
print("bz2file_seek_modes OK")
