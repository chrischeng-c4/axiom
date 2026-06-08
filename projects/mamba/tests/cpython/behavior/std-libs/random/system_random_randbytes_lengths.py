# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_randbytes_lengths"
# subject = "random.SystemRandom.randbytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.randbytes: SystemRandom.randbytes(n) returns exactly n bytes for n in 1..9 and randbytes(0) == b''"""
import random

gen = random.SystemRandom()
for n in range(1, 10):
    data = gen.randbytes(n)
    assert type(data) is bytes and len(data) == n, f"randbytes({n}) = {data!r}"
assert gen.randbytes(0) == b"", "randbytes(0)"

print("system_random_randbytes_lengths OK")
