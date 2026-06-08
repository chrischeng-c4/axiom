# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "compress_returns_bytes"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: lzma.compress returns a bytes object whose .xz stream begins with the 6-byte magic header"""
import lzma


data = b"hello xz world " * 20
c = lzma.compress(data)
assert isinstance(c, bytes), f"compress returns bytes: {type(c)!r}"
# .xz stream header: 6 bytes 0xFD '7' 'z' 'X' 'Z' 0x00.
assert c[:6] == b"\xfd7zXZ\x00", f"xz magic = {c[:6]!r}"
print("compress_returns_bytes OK")
