# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "zlib_codec_compresses"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the zlib_codec compresses then losslessly restores: a 20x-repeated payload encodes to a strictly smaller bytes object and decodes back equal"""
import codecs

_data = b"compress me please " * 20
_z = codecs.encode(_data, "zlib_codec")
assert isinstance(_z, bytes), "zlib encoded is bytes"
assert len(_z) < len(_data), "zlib compressed smaller"
_back = codecs.decode(_z, "zlib_codec")
assert _back == _data, f"zlib round-trip = {_back!r}"

print("zlib_codec_compresses OK")
