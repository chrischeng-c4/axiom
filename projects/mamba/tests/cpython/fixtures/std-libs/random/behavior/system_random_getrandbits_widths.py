# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_getrandbits_widths"
# subject = "random.SystemRandom.getrandbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.getrandbits: SystemRandom.getrandbits(k) yields a k-bit non-negative integer for k in {1,8,32,64,128} and getrandbits(0) == 0"""
import random

gen = random.SystemRandom()
for k in (1, 8, 32, 64, 128):
    v = gen.getrandbits(k)
    assert 0 <= v < 2 ** k, f"getrandbits({k}) = {v!r}"
assert gen.getrandbits(0) == 0, "getrandbits(0)"

print("system_random_getrandbits_widths OK")
