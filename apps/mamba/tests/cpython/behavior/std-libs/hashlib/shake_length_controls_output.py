# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_length_controls_output"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: the requested length sizes the output: shake_128.digest(8) is 8 bytes, hexdigest(8) is 16 hex chars, shake_256.digest(100) is 100 bytes"""
import hashlib

assert len(hashlib.shake_128(b"x").digest(8)) == 8, "shake digest(8) is 8 bytes"
assert len(hashlib.shake_128(b"x").hexdigest(8)) == 16, "shake hexdigest(8) is 16 hex chars"
assert len(hashlib.shake_256(b"x").digest(100)) == 100, "shake digest(100) is 100 bytes"

print("shake_length_controls_output OK")
