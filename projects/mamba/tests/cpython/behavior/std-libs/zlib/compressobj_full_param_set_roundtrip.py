# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compressobj_full_param_set_roundtrip"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: compressobj accepts the full positional parameter set (level, method, negative wbits, memLevel, strategy) and the matching raw decompressor recovers the input"""
import zlib

_TEXT = b"the quick brown fox jumps over the lazy dog " * 64
_co = zlib.compressobj(2, zlib.DEFLATED, -12, 9, zlib.Z_FILTERED)
_blob = _co.compress(_TEXT) + _co.flush()
_dco = zlib.decompressobj(-12)
assert _dco.decompress(_blob) + _dco.flush() == _TEXT, "full-param round-trip"

print("compressobj_full_param_set_roundtrip OK")
