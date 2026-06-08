# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_output_prefix_stable"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: the XOF stream is prefix-stable: a 32-byte shake_128 output starts with the same bytes as the 8-byte output of the same input"""
import hashlib

_short = hashlib.shake_128(b"prefix").digest(8)
_long = hashlib.shake_128(b"prefix").digest(32)
assert _long[:8] == _short, "shake output is prefix-stable"

print("shake_output_prefix_stable OK")
