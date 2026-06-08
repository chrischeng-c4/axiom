# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_primary_hues"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: the HSV hue of pure red is 0, pure green is 1/3, pure blue is 2/3"""
import colorsys

EPS = 1e-9
assert abs(colorsys.rgb_to_hsv(1, 0, 0)[0] - 0.0) < EPS, "red hue"
assert abs(colorsys.rgb_to_hsv(0, 1, 0)[0] - 1.0 / 3) < EPS, "green hue"
assert abs(colorsys.rgb_to_hsv(0, 0, 1)[0] - 2.0 / 3) < EPS, "blue hue"

print("hsv_primary_hues OK")
