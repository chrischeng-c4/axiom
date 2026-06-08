# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_properties"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: a NormalDist derives mean/median/mode == mu, stdev == sigma, and variance == sigma**2 (NormalDist(100,15) -> variance 225)"""
from statistics import NormalDist

# mean / median / mode all equal mu; stdev equals sigma; variance == sigma**2.
X = NormalDist(100, 15)
assert X.mean == 100 and X.median == 100 and X.mode == 100
assert X.stdev == 15
assert X.variance == 225, X.variance

print("normaldist_properties OK")
