# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_bytesio_roundtrip"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: GzipFile over a BytesIO fileobj writes in 'wb' and reads back the same bytes in 'rb'"""
import gzip
import io

_buf = io.BytesIO()
with gzip.GzipFile(fileobj=_buf, mode="wb") as _w:
    _w.write(b"buffer data")
_buf.seek(0)
with gzip.GzipFile(fileobj=_buf, mode="rb") as _r:
    _out = _r.read()
assert _out == b"buffer data", f"BytesIO GzipFile = {_out!r}"

print("gzipfile_bytesio_roundtrip OK")
