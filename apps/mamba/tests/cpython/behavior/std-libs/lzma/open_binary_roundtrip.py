# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "open_binary_roundtrip"
# subject = "lzma.open"
# kind = "semantic"
# xfail = "lzma.open is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:81-82)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.open: lzma.open(buf, 'wb') write then lzma.open(buf, 'rb') read round-trips the bytes through a BytesIO"""
import lzma


import io
buf = io.BytesIO()
with lzma.open(buf, "wb") as f:
    f.write(b"line A\n")
    f.write(b"line B\n")
buf.seek(0)
with lzma.open(buf, "rb") as f:
    content = f.read()
assert content == b"line A\nline B\n", f"lzma.open = {content!r}"
print("open_binary_roundtrip OK")
