# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2file_wrap_bytesio_stays_open"
# subject = "bz2.BZ2File"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2File: closing a BZ2File that wraps a BytesIO leaves the underlying object open and the bytes valid"""
import bz2
import io

outer = io.BytesIO()
with bz2.BZ2File(outer, "w") as f:
    f.write(b"payload")
assert outer.closed is False, "underlying BytesIO stays open"
assert bz2.decompress(outer.getvalue()) == b"payload", "wrapped write round-trip"
print("bz2file_wrap_bytesio_stays_open OK")
