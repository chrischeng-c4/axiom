# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "raw_format_delta_chain_roundtrip"
# subject = "lzma.compress"
# kind = "semantic"
# xfail = "lzma.compress/decompress ignore format and filters kwargs; FORMAT_RAW filter chains not honored (src/runtime/stdlib/lzma_mod.rs:153,179)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.compress: a chained DELTA+LZMA2 filter list under FORMAT_RAW round-trips through compress and decompress"""
import lzma


text = b"the same line repeated for compressibility\n" * 40
filters_chain = [
    {"id": lzma.FILTER_DELTA, "dist": 2},
    {"id": lzma.FILTER_LZMA2, "preset": lzma.PRESET_DEFAULT | lzma.PRESET_EXTREME},
]
raw = lzma.compress(text, format=lzma.FORMAT_RAW, filters=filters_chain)
back = lzma.decompress(raw, format=lzma.FORMAT_RAW, filters=filters_chain)
assert back == text, "DELTA+LZMA2 chain round-trip"
print("raw_format_delta_chain_roundtrip OK")
