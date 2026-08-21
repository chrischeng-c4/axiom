# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "module_helpers_share_default_generator"
# subject = "random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.seed: module-level helpers share one default generator: random.seed(2024) then [random.random() for _ in range(5)] replays identically after re-seeding"""
import random

random.seed(2024)
a = [random.random() for _ in range(5)]
random.seed(2024)
b = [random.random() for _ in range(5)]
assert a == b, f"module reseed replay: {a!r} != {b!r}"

print("module_helpers_share_default_generator OK")
