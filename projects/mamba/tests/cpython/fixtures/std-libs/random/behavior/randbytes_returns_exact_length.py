# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randbytes_returns_exact_length"
# subject = "random.randbytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randbytes: module-level randbytes(n) returns exactly n bytes (a bytes object) and randbytes(0) returns b''"""
import random

random.seed(0)
data = random.randbytes(8)
assert type(data) is bytes and len(data) == 8, f"randbytes = {data!r}"
assert random.randbytes(0) == b"", "randbytes(0)"

print("randbytes_returns_exact_length OK")
