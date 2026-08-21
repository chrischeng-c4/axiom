# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "uu_codec_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the uu_codec is a bytes->bytes transform via codecs.encode/decode: b'hello world' encodes to bytes and decodes back equal"""
import codecs

_data = b"hello world"
_uu = codecs.encode(_data, "uu_codec")
assert isinstance(_uu, bytes), "uu_codec yields bytes"
assert codecs.decode(_uu, "uu_codec") == _data, "uu round-trip"

print("uu_codec_roundtrip OK")
