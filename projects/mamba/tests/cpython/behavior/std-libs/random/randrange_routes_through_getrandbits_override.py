# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randrange_routes_through_getrandbits_override"
# subject = "random.Random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.randrange: when a subclass overrides getrandbits, randrange routes through it: calling randrange(42) on the subclass records that getrandbits was invoked"""
import random

called = set()

class OverrideGetrandbits(random.Random):
    def getrandbits(self, n):
        called.add("getrandbits")
        return random.Random.getrandbits(self, n)

OverrideGetrandbits().randrange(42)
assert called == {"getrandbits"}, f"getrandbits dispatch: {called!r}"

print("randrange_routes_through_getrandbits_override OK")
