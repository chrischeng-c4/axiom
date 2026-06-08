# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "bytesio_getvalue_returns_bytes"
# subject = "io.BytesIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BytesIO: BytesIO.getvalue() returns a bytes object equal to the written content"""
import io

_bv = io.BytesIO(b"\x01\x02")
assert isinstance(_bv.getvalue(), bytes), "getvalue returns bytes"
assert _bv.getvalue() == b"\x01\x02", f"BytesIO getvalue = {_bv.getvalue()!r}"

print("bytesio_getvalue_returns_bytes OK")
