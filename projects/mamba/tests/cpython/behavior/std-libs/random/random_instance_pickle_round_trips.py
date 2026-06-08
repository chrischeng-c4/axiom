# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "random_instance_pickle_round_trips"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: pickling a Random preserves generator state: a dumps/loads round-trip reproduces the same 10-draw stream as the original"""
import random

import pickle

src = random.Random(7)
blob = pickle.dumps(src)
orig = [src.random() for _ in range(10)]
restored = pickle.loads(blob)
assert orig == [restored.random() for _ in range(10)], "pickle replay differs"

print("random_instance_pickle_round_trips OK")
