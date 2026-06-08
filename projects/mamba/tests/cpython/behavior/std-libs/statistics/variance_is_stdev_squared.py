# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "variance_is_stdev_squared"
# subject = "statistics.variance"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.variance: sample variance equals the square of the sample stdev for the same data, with the exact CPython value 4.571428571428571"""
import statistics

_data = [2, 4, 4, 4, 5, 5, 7, 9]
_s = statistics.stdev(_data)
_v = statistics.variance(_data)
# variance == stdev ** 2.
assert abs(_v - _s ** 2) < 1e-10, (_v, _s)
assert abs(_v - 4.571428571428571) < 1e-10, _v
# pvariance for 1..5 is exactly 2.0.
assert abs(statistics.pvariance([1, 2, 3, 4, 5]) - 2.0) < 1e-10

print("variance_is_stdev_squared OK")
