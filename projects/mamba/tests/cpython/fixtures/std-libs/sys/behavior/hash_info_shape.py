# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "hash_info_shape"
# subject = "sys.hash_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.hash_info: hash_info has 9 fields, a modulus that fits in 'width' bits, and a known algorithm name (fnv / siphash13 / siphash24)"""
import sys

assert len(sys.hash_info) == 9, f"hash_info len = {len(sys.hash_info)!r}"
assert sys.hash_info.modulus < 2 ** sys.hash_info.width, \
    "modulus fits within width bits"
assert sys.hash_info.algorithm in ("fnv", "siphash13", "siphash24"), \
    f"hash algorithm = {sys.hash_info.algorithm!r}"
print("hash_info_shape OK")
