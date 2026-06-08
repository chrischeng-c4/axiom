# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "empty_bytes_roundtrip"
# subject = "gzip.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.decompress: compressing then decompressing empty bytes round-trips to empty bytes"""
import gzip

_empty = gzip.compress(b"")
assert gzip.decompress(_empty) == b"", "empty round-trip"

print("empty_bytes_roundtrip OK")
