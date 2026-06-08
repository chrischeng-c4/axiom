# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "gammavariate_rejects_nonpositive_params"
# subject = "random.Random.gammavariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.gammavariate: gammavariate rejects non-positive alpha or beta with ValueError: (-1,3), (0,2), (2,0), (1,-3) all raise"""
import random

gen = random.Random(0)
for args in [(-1, 3), (0, 2), (2, 0), (1, -3)]:
    try:
        gen.gammavariate(*args)
        raise AssertionError(f"expected ValueError for gammavariate{args}")
    except ValueError:
        pass

print("gammavariate_rejects_nonpositive_params OK")
