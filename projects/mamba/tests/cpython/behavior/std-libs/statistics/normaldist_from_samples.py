# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_from_samples"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: NormalDist.from_samples reconstructs an estimate (from_samples([96,107,90,92,110]) == NormalDist(99,9)) and samples(seed=...) is reproducible for a fixed seed and differs across seeds"""
from statistics import NormalDist, mean

# from_samples reconstructs an estimate from the data.
assert NormalDist.from_samples([96, 107, 90, 92, 110]) == NormalDist(99, 9)
# samples(seed=...) is reproducible for a fixed seed and differs across seeds.
S = NormalDist(10000, 3.0)
data = S.samples(500)
assert len(data) == 500 and set(map(type, data)) == {float}
assert abs(mean(data) - 10000) < 3.0 * 8
assert S.samples(50, seed="alpha") == S.samples(50, seed="alpha")
assert S.samples(50, seed="alpha") != S.samples(50, seed="beta")

print("normaldist_from_samples OK")
