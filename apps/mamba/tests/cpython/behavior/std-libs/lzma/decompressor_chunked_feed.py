# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "decompressor_chunked_feed"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "LZMADecompressor is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: feeding an XZ stream in fixed-size chunks reassembles the original; eof stays False until the stream ends then becomes True"""
import lzma


text = b"To be, or not to be, that is the question.\n" * 60
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
lzd = lzma.LZMADecompressor()
out = []
for i in range(0, len(xz), 10):
    assert not lzd.eof, "eof stays False until the stream ends"
    out.append(lzd.decompress(xz[i:i + 10]))
assert b"".join(out) == text, "chunked decompress reassembles input"
assert lzd.eof is True, "eof True once the stream is consumed"
print("decompressor_chunked_feed OK")
