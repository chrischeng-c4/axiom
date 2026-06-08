# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "decompressobj_max_length_caps_via_unconsumed_tail"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: a max_length argument caps output bytes, the remaining input reappears in unconsumed_tail, and resuming from unconsumed_tail reconstructs the full payload"""
import zlib

_src = zlib.compress(b"abcdefghijklmnopqrstuvwxyz")
_dco = zlib.decompressobj()
_head = _dco.decompress(_src, 5)
assert _head == b"abcde", "max_length caps output to 5 bytes"
assert len(_dco.unconsumed_tail) > 0, "leftover input in unconsumed_tail"
_tail = _dco.decompress(_dco.unconsumed_tail)
assert _head + _tail == b"abcdefghijklmnopqrstuvwxyz", "resume from unconsumed_tail"

print("decompressobj_max_length_caps_via_unconsumed_tail OK")
