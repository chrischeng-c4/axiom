# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_seek_tell"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: seek/tell work on a read file; seeking past the end clamps to the stream length and a following read() returns b''"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
xz = lzma.compress(text, format=lzma.FORMAT_XZ)
with lzma.LZMAFile(BytesIO(xz)) as f:
    assert f.seekable() is True, "read file is seekable"
    f.seek(100)
    assert f.tell() == 100, "tell reports position"
    assert f.read() == text[100:], "read resumes after seek"
    f.seek(len(text) + 5000)
    assert f.tell() == len(text), "seek past end clamps to length"
    assert f.read() == b"", "nothing left after end"
print("lzmafile_seek_tell OK")
