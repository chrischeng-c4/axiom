# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf8_multibyte_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: codecs.encode/decode round-trips a multi-byte string: 'hello 日本語 café' encodes to bytes and decodes back equal under utf-8"""
import codecs

_text = "hello 日本語 café"
_encoded = codecs.encode(_text, "utf-8")
assert isinstance(_encoded, bytes), f"encode returns bytes: {type(_encoded)!r}"
_decoded = codecs.decode(_encoded, "utf-8")
assert _decoded == _text, f"utf-8 round-trip = {_decoded!r}"

print("utf8_multibyte_roundtrip OK")
