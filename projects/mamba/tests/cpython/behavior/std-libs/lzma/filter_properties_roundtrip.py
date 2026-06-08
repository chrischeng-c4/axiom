# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "filter_properties_roundtrip"
# subject = "lzma._encode_filter_properties"
# kind = "semantic"
# xfail = "lzma._encode_filter_properties / _decode_filter_properties are not implemented (src/runtime/stdlib/lzma_mod.rs)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma._encode_filter_properties: _encode_filter_properties for FILTER_LZMA1 produces bytes that _decode_filter_properties decodes back to the same id and pb/lp/lc"""
import lzma


props = lzma._encode_filter_properties(
    {"id": lzma.FILTER_LZMA1, "pb": 2, "lp": 0, "lc": 3, "dict_size": 8 << 20}
)
assert isinstance(props, bytes), "encoded properties are bytes"
spec = lzma._decode_filter_properties(lzma.FILTER_LZMA1, props)
assert spec["id"] == lzma.FILTER_LZMA1, "decoded id"
assert spec["pb"] == 2 and spec["lp"] == 0 and spec["lc"] == 3, "decoded pb/lp/lc"
assert spec["dict_size"] == 8 << 20, "decoded dict_size"
print("filter_properties_roundtrip OK")
