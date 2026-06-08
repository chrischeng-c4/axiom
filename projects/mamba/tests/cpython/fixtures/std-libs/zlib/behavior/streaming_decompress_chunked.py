# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "streaming_decompress_chunked"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: feeding a compressed stream to decompressobj in fixed-size chunks then flushing reassembles the original input"""
import zlib

_original = b"decomp test " * 50
_compressed = zlib.compress(_original)
_decomp = zlib.decompressobj()
_result = b""
_chunk = 20
for _i in range(0, len(_compressed), _chunk):
    _result += _decomp.decompress(_compressed[_i:_i + _chunk])
_result += _decomp.flush()
assert _result == _original, "streaming decompress"

print("streaming_decompress_chunked OK")
