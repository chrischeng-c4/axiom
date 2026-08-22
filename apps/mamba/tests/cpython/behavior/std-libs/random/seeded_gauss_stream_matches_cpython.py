# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# module = "random"
# dimension = "behavior"
# case = "seeded_gauss_stream_matches_cpython"
# subject = "random.Random.gauss"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.gauss: seeded streams and per-instance cached variates match CPython."""
import random


seeded_1234 = random.Random(1234)
assert seeded_1234.gauss() == 1.0542196419272387
assert seeded_1234.gauss() == -0.22555725575068641
assert seeded_1234.gauss() == 2.1970405483761803
assert seeded_1234.gauss() == 0.1034917897273693

seeded_42 = random.Random(42)
assert seeded_42.gauss() == -0.14409032957792836

left = random.Random(1234)
right = random.Random(1234)
assert (left.gauss(), right.gauss(), left.gauss(), right.gauss()) == (
    1.0542196419272387,
    1.0542196419272387,
    -0.22555725575068641,
    -0.22555725575068641,
)

print("seeded_gauss_stream_matches_cpython OK")
