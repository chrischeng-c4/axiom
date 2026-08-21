# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "bytesio_write_read_roundtrip"
# subject = "io.BytesIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.BytesIO: BytesIO writes and reads bytes; seek(0) then read() returns the written bytes"""
import io

_bbuf = io.BytesIO()
_bbuf.write(b"bytes data")
_bbuf.seek(0)
assert _bbuf.read() == b"bytes data", "BytesIO round-trip"

print("bytesio_write_read_roundtrip OK")
