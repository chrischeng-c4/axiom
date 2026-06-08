# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "bcj_filter_id_only_decode"
# subject = "lzma._decode_filter_properties"
# kind = "semantic"
# xfail = "lzma._decode_filter_properties and FILTER_ARM/FILTER_SPARC are not implemented (src/runtime/stdlib/lzma_mod.rs)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma._decode_filter_properties: BCJ filters (X86/ARM/SPARC) decode zero-length properties to an id-only spec {'id': fid}"""
import lzma


for fid in (lzma.FILTER_X86, lzma.FILTER_ARM, lzma.FILTER_SPARC):
    only_id = lzma._decode_filter_properties(fid, b"")
    assert only_id == {"id": fid}, f"id-only decode {fid}"
print("bcj_filter_id_only_decode OK")
