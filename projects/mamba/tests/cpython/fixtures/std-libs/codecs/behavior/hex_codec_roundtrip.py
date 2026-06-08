# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "hex_codec_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the hex_codec maps bytes to lowercase hex: b'\\xde\\xad\\xbe\\xef' encodes to b'deadbeef' and decodes back"""
import codecs

_data = b"\xde\xad\xbe\xef"
_hex = codecs.encode(_data, "hex_codec")
assert _hex == b"deadbeef", f"hex_codec = {_hex!r}"
_back = codecs.decode(_hex, "hex_codec")
assert _back == _data, f"hex decode = {_back!r}"

print("hex_codec_roundtrip OK")
