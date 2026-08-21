# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "format_xz_auto_detected"
# subject = "lzma.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.decompress: an explicit FORMAT_XZ stream is decoded by the default FORMAT_AUTO decompress"""
import lzma


c = lzma.compress(b"auto-detect me", format=lzma.FORMAT_XZ)
assert lzma.decompress(c) == b"auto-detect me", "FORMAT_AUTO detects XZ"
print("format_xz_auto_detected OK")
