# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "streaming_compress_concatenates"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: feeding a payload to compressobj in successive chunks then flushing yields a stream that one-shot decompress recovers to the full input"""
import zlib

_data = b"stream test data " * 100
_comp = zlib.compressobj(level=6)
_parts = [_data[:50], _data[50:150], _data[150:]]
_compressed = b""
for _part in _parts:
    _compressed += _comp.compress(_part)
_compressed += _comp.flush()
assert zlib.decompress(_compressed) == _data, "streaming compress"

print("streaming_compress_concatenates OK")
