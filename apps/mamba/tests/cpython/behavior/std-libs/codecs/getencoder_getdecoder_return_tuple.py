# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "getencoder_getdecoder_return_tuple"
# subject = "codecs.getencoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getencoder: getencoder('ascii')('world') returns (b'world', 5) and getdecoder('ascii')(b'world') returns ('world', 5) — value plus consumed length"""
import codecs

_encoder = codecs.getencoder("ascii")
_result, _n = _encoder("world")
assert _result == b"world", f"encoder result = {_result!r}"
assert _n == 5, f"encoder n = {_n!r}"
_decoder = codecs.getdecoder("ascii")
_str, _m = _decoder(b"world")
assert _str == "world", f"decoder str = {_str!r}"
assert _m == 5, f"decoder m = {_m!r}"

print("getencoder_getdecoder_return_tuple OK")
