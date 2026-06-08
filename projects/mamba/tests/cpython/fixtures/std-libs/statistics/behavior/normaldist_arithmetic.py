# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_arithmetic"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: translating/scaling a NormalDist shifts/scales it (X+10, 10*X, X/10), adding two NormalDists combines variances (sigma=sqrt(12**2+5**2)=13), and unary negation flips the mean and copies"""
from statistics import NormalDist

X = NormalDist(100, 15)
# Translation shifts the mean; scaling scales both mean and sigma.
assert X + 10 == NormalDist(110, 15) and 10 + X == NormalDist(110, 15)
assert X - 10 == NormalDist(90, 15)
assert X * 10 == NormalDist(1000, 150) and X / 10 == NormalDist(10, 1.5)
# Same-type add/sub combine variances: sigma = sqrt(12**2 + 5**2) = 13.
assert NormalDist(100, 12) + NormalDist(40, 5) == NormalDist(140, 13)
assert NormalDist(100, 12) - NormalDist(40, 5) == NormalDist(60, 13)
# Unary negation flips the mean, keeps sigma, and produces a fresh object.
neg = -X
assert neg is not X and neg.mean == -100 and neg.stdev == 15

print("normaldist_arithmetic OK")
