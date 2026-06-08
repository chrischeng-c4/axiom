# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "getstate_setstate_round_trips"
# subject = "random.Random.getstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.getstate: getstate snapshots the generator and setstate rewinds it exactly: draws taken before and after restoring the snapshot are equal"""
import random

gen = random.Random(12345)

# getstate snapshots the generator; setstate rewinds it exactly.
snapshot = gen.getstate()
before = [gen.random() for _ in range(5)]
gen.setstate(snapshot)
after = [gen.random() for _ in range(5)]
assert before == after, f"setstate replay: {before!r} != {after!r}"

print("getstate_setstate_round_trips OK")
