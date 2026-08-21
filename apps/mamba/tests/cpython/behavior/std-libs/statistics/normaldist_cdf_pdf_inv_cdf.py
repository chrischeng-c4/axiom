# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_cdf_pdf_inv_cdf"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: the standard normal has cdf(0) == 0.5, a pdf peaking at the mean, and inv_cdf inverts cdf; zscore standardises a value (NormalDist(100,15).zscore(142) == 2.8)"""
from statistics import NormalDist

# The standard normal: cdf(0) == 0.5 and the pdf peaks at the mean.
Z = NormalDist()
assert abs(Z.cdf(0.0) - 0.5) < 1e-12, Z.cdf(0.0)
assert Z.pdf(0.0) > Z.pdf(1.0)
# inv_cdf inverts cdf.
assert abs(Z.inv_cdf(Z.cdf(0.7)) - 0.7) < 1e-9
# zscore standardises a value relative to (mu, sigma).
X = NormalDist(100, 15)
assert X.zscore(142) == 2.8 and X.zscore(58) == -2.8 and X.zscore(100) == 0.0

print("normaldist_cdf_pdf_inv_cdf OK")
