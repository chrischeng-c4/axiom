# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "yiq_red_luminance"
# subject = "colorsys.rgb_to_yiq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: the YIQ luminance (Y) of pure red is the documented 0.3 coefficient"""
import colorsys

EPS = 1e-9
y, i, q = colorsys.rgb_to_yiq(1, 0, 0)
assert abs(y - 0.3) < EPS, ("red Y", y)

print("yiq_red_luminance OK")
