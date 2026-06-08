# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "getrandbits_returns_bounded_int"
# subject = "random.getrandbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.getrandbits: getrandbits(16) returns an int in [0, 2**16): the result is an int and 0 <= bits < 65536"""
import random

random.seed(8)
bits = random.getrandbits(16)
assert isinstance(bits, int), f"getrandbits type = {type(bits)!r}"
assert 0 <= bits < 2 ** 16, f"getrandbits 16-bit: {bits!r}"

print("getrandbits_returns_bounded_int OK")
