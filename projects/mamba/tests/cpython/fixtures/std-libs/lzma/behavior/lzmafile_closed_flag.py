# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_closed_flag"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: an LZMAFile reports closed False while open and True after close()"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
f = lzma.LZMAFile(BytesIO(xz))
assert f.closed is False, "open file not closed"
f.read()
f.close()
assert f.closed is True, "closed after close()"
print("lzmafile_closed_flag OK")
