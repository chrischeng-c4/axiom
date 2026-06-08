# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mixin_order_decides_dispatch"
# subject = "random.Random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.randrange: MRO order decides which override wins: a subclass mixing both getrandbits and random overrides uses getrandbits for randrange when it appears first"""
import random

called = set()

class RandomMixin:
    def random(self):
        called.add("mixin.random")
        return random.Random.random(self)

class GetrandbitsMixin:
    def getrandbits(self, n):
        called.add("mixin.getrandbits")
        return random.Random.getrandbits(self, n)

class FromGetrandbits(GetrandbitsMixin, RandomMixin, random.Random):
    pass

FromGetrandbits().randrange(42)
assert called == {"mixin.getrandbits"}, f"mixin dispatch: {called!r}"

print("mixin_order_decides_dispatch OK")
