# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "lzmafile_append_multistream"
# subject = "lzma.LZMAFile"
# kind = "semantic"
# xfail = "LZMAFile is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:79-80)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMAFile: appending opens a fresh stream so two writes concatenate into one multistream file that reads back as the joined input"""
import lzma


from io import BytesIO
text = b"Now is the winter of our discontent.\n" * 50
dst = BytesIO()
with lzma.LZMAFile(dst, "w") as f:
    f.write(text[:100])
with lzma.LZMAFile(dst, "a") as f:
    f.write(text[100:])
with lzma.LZMAFile(BytesIO(dst.getvalue())) as f:
    assert f.read() == text, "append round-trip"
print("lzmafile_append_multistream OK")
