# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_value_table"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: the canonical CPython RGB<->HSV value table (black/blue/green/cyan/red/purple/yellow/white/grey) matches in both directions"""
import colorsys

EPS = 1e-7
# (rgb, hsv) — straight from CPython's test_colorsys.test_hsv_values.
table = [
    ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),         # black
    ((0.0, 0.0, 1.0), (4. / 6., 1.0, 1.0)),     # blue
    ((0.0, 1.0, 0.0), (2. / 6., 1.0, 1.0)),     # green
    ((0.0, 1.0, 1.0), (3. / 6., 1.0, 1.0)),     # cyan
    ((1.0, 0.0, 0.0), (0.0, 1.0, 1.0)),         # red
    ((1.0, 0.0, 1.0), (5. / 6., 1.0, 1.0)),     # purple
    ((1.0, 1.0, 0.0), (1. / 6., 1.0, 1.0)),     # yellow
    ((1.0, 1.0, 1.0), (0.0, 0.0, 1.0)),         # white
    ((0.5, 0.5, 0.5), (0.0, 0.0, 0.5)),         # grey
]
for rgb, hsv in table:
    got = colorsys.rgb_to_hsv(*rgb)
    for a, b in zip(got, hsv):
        assert abs(a - b) < EPS, ("rgb->hsv", rgb, got)
    back = colorsys.hsv_to_rgb(*hsv)
    for a, b in zip(back, rgb):
        assert abs(a - b) < EPS, ("hsv->rgb", hsv, back)

print("hsv_value_table OK")
