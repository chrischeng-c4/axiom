# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "system_random_getrandbits_validates_arg"
# subject = "random.SystemRandom.getrandbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.getrandbits: SystemRandom.getrandbits validates its argument: getrandbits(-1) raises ValueError and getrandbits(10.1) raises TypeError"""
import random

gen = random.SystemRandom()
for bad in [(-1, ValueError), (10.1, TypeError)]:
    arg, exc = bad
    try:
        gen.getrandbits(arg)
        raise AssertionError(f"expected {exc.__name__} for getrandbits({arg!r})")
    except exc:
        pass

print("system_random_getrandbits_validates_arg OK")
