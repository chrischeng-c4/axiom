# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_peek_then_read"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: peek() returns a prefix of the stream without advancing the read position, so a following read() still returns everything"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
with lzma.LZMAFile(BytesIO(xz)) as f:
    peeked = f.peek()
    assert len(peeked) > 0, "peek returns data"
    assert text.startswith(peeked), "peek is a prefix of the stream"
    assert f.read() == text, "read still returns full stream after peek"
print("lzmafile_peek_then_read OK")
