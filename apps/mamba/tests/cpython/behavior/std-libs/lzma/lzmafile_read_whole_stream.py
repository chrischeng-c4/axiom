# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_read_whole_stream"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: LZMAFile.read() returns the whole decoded stream and a second read() yields b''"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
with lzma.LZMAFile(BytesIO(xz)) as f:
    assert f.read() == text, "read whole xz stream"
    assert f.read() == b"", "read past end -> empty"
print("lzmafile_read_whole_stream OK")
