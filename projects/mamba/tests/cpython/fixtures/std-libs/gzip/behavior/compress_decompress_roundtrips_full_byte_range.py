# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_decompress_roundtrips_full_byte_range"
# subject = "gzip.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.decompress: compress then decompress round-trips an arbitrary payload covering the full 0..255 byte range repeated 10 times, byte-for-byte"""
import gzip

_payload = bytes(range(256)) * 10
_compressed = gzip.compress(_payload)
assert gzip.decompress(_compressed) == _payload, "full byte range round-trip"

print("compress_decompress_roundtrips_full_byte_range OK")
