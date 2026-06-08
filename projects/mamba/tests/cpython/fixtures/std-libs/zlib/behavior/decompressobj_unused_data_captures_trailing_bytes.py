# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "decompressobj_unused_data_captures_trailing_bytes"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: bytes appended after a complete deflate stream land in unused_data (not in the output), and unconsumed_tail is empty without a max_length cap"""
import zlib

_packed = zlib.compress(b"abcdefghijklmnopqrstuvwxyz") + b"0123456789"
_dco = zlib.decompressobj()
assert _dco.decompress(_packed) == b"abcdefghijklmnopqrstuvwxyz", "stops at stream end"
assert _dco.unused_data == b"0123456789", "trailing bytes in unused_data"
assert _dco.unconsumed_tail == b"", "no unconsumed tail without max_length"

print("decompressobj_unused_data_captures_trailing_bytes OK")
