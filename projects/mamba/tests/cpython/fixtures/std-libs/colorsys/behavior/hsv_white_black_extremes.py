# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_white_black_extremes"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: white (1,1,1) maps to HSV saturation 0 / value 1 and black (0,0,0) maps to value 0"""
import colorsys

EPS = 1e-9
wh, ws, wv = colorsys.rgb_to_hsv(1.0, 1.0, 1.0)
assert abs(ws) < EPS, "white saturation"
assert abs(wv - 1.0) < EPS, "white value"

bh, bs, bv = colorsys.rgb_to_hsv(0.0, 0.0, 0.0)
assert abs(bv) < EPS, "black value"

print("hsv_white_black_extremes OK")
