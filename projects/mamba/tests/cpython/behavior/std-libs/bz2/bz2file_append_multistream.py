# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_append_multistream"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: appending (ab mode) concatenates bzip2 streams and a reader sees the two payloads joined in order"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"foo")
with bz2.BZ2File(buf, "ab") as f:
    f.write(b"bar")
buf.seek(0)
with bz2.BZ2File(buf, "rb") as f:
    assert f.read() == b"foobar", "append/multi-stream ordering"
print("bz2file_append_multistream OK")
