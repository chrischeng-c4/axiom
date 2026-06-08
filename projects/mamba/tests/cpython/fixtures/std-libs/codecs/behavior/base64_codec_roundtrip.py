# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "base64_codec_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the base64_codec is a bytes->bytes transform reached via codecs.encode/decode: b'hello world' round-trips through base64_codec"""
import codecs

_data = b"hello world"
_b64 = codecs.encode(_data, "base64_codec")
assert isinstance(_b64, bytes), f"base64 codec returns bytes: {type(_b64)!r}"
_decoded = codecs.decode(_b64, "base64_codec")
assert _decoded == _data, f"base64 round-trip = {_decoded!r}"

print("base64_codec_roundtrip OK")
