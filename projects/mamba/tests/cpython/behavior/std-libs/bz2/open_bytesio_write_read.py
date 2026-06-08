# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_bytesio_write_read"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open on a BytesIO round-trips bytes written in wb mode back through rb mode"""
import bz2
import io

buf = io.BytesIO()
with bz2.open(buf, "wb") as f:
    f.write(b"line 1\n")
    f.write(b"line 2\n")
buf.seek(0)
with bz2.open(buf, "rb") as f:
    content = f.read()
assert content == b"line 1\nline 2\n", f"bz2.open write/read = {content!r}"
print("open_bytesio_write_read OK")
