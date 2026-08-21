# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_closed_raises_valueerror"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: operating on a closed BZ2File raises ValueError and closed is True after the context exits"""
import bz2
import io

buf = io.BytesIO()
with bz2.BZ2File(buf, "wb") as f:
    f.write(b"0123456789abcdef")
assert f.closed is True, "closed after context exit"
buf.seek(0)
closed = bz2.BZ2File(buf, "rb")
closed.close()
try:
    closed.read()
    raise AssertionError("expected ValueError on closed file")
except ValueError:
    pass
print("bz2file_closed_raises_valueerror OK")
