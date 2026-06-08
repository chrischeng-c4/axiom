# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_gray_zero_saturation"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: any gray (r==g==b) has HSV saturation 0 across the [0,1] value range"""
import colorsys

EPS = 1e-9
for v in [0.0, 0.25, 0.5, 0.75, 1.0]:
    h, s, val = colorsys.rgb_to_hsv(v, v, v)
    assert abs(s) < EPS, ("gray saturation", v, s)

print("hsv_gray_zero_saturation OK")
