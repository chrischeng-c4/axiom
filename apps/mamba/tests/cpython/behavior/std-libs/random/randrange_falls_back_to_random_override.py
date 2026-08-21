# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randrange_falls_back_to_random_override"
# subject = "random.Random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.randrange: a subclass overriding only random() makes randrange fall back to random(): calling randrange(42) records that random was invoked"""
import random

called = set()

class OverrideRandom(random.Random):
    def random(self):
        called.add("random")
        return random.Random.random(self)

OverrideRandom().randrange(42)
assert called == {"random"}, f"random dispatch: {called!r}"

print("randrange_falls_back_to_random_override OK")
