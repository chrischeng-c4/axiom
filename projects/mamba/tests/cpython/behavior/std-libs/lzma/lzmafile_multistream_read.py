# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_multistream_read"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: LZMAFile decodes concatenated streams as one logical file"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
with lzma.LZMAFile(BytesIO(xz * 3)) as f:
    assert f.read() == text * 3, "multistream read"
print("lzmafile_multistream_read OK")
