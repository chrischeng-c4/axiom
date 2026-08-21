# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_write_matches_compress"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: writing through an LZMAFile writer produces a stream byte-identical to lzma.compress, and the writer is writable but not readable"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
dst = BytesIO()
with lzma.LZMAFile(dst, "w") as f:
    assert f.writable() is True, "write file is writable"
    assert f.readable() is False, "write file is not readable"
    f.write(text)
assert dst.getvalue() == lzma.compress(text), "LZMAFile write == lzma.compress"
print("lzmafile_write_matches_compress OK")
