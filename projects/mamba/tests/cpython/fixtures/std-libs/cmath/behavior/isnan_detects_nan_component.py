# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "isnan_detects_nan_component"
# subject = "cmath.isnan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isnan: isnan is True when a component is NaN and False for finite int/complex; accepts float('nan')"""
import cmath

assert cmath.isnan(complex(float("nan"), 0)), "isnan real nan"
assert cmath.isnan(float("nan")), "isnan(float nan)"
assert not cmath.isnan(1), "isnan(int 1) is False"
print("isnan_detects_nan_component OK")
