# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "raw_format_lzma2_roundtrip"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = "lzma.compress/decompress ignore format and filters kwargs; FORMAT_RAW not honored (src/runtime/stdlib/lzma_mod.rs:153,179)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: FORMAT_RAW with an explicit FILTER_LZMA2 filter chain round-trips on both compress and decompress"""
import lzma


text = b"the same line repeated for compressibility\n" * 40
filters_lzma2 = [{"id": lzma.FILTER_LZMA2, "preset": 3}]
raw = lzma.compress(text, format=lzma.FORMAT_RAW, filters=filters_lzma2)
back = lzma.decompress(raw, format=lzma.FORMAT_RAW, filters=filters_lzma2)
assert back == text, "FORMAT_RAW LZMA2 round-trip"
print("raw_format_lzma2_roundtrip OK")
