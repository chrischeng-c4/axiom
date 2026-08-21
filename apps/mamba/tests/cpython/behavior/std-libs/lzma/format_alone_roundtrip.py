# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "format_alone_roundtrip"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = "lzma.compress/decompress ignore the format kwarg; FORMAT_ALONE not honored (src/runtime/stdlib/lzma_mod.rs:153,179)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: the legacy FORMAT_ALONE (LZMA1) format compresses and decompresses round-trip"""
import lzma


c = lzma.compress(b"legacy format test", format=lzma.FORMAT_ALONE)
assert isinstance(c, bytes), "FORMAT_ALONE returns bytes"
d = lzma.decompress(c, format=lzma.FORMAT_ALONE)
assert d == b"legacy format test", f"FORMAT_ALONE round-trip = {d!r}"
print("format_alone_roundtrip OK")
