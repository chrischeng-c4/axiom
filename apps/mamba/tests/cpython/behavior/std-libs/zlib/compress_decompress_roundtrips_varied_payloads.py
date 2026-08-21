# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_decompress_roundtrips_varied_payloads"
# subject = "zlib.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompress: compress then decompress round-trips a range of payloads byte-for-byte: empty, single byte, the full 0..255 byte range, a 10000-byte repeat, and repeated text"""
import zlib

_payloads = [
    b"",
    b"x",
    bytes(range(256)),
    b"a" * 10000,
    b"hello world " * 100,
]
for _p in _payloads:
    _rt = zlib.decompress(zlib.compress(_p))
    assert _rt == _p, f"round-trip len={len(_p)}"

print("compress_decompress_roundtrips_varied_payloads OK")
