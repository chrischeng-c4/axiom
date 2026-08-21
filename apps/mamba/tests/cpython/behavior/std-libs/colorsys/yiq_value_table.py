# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "yiq_value_table"
# subject = "colorsys.rgb_to_yiq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: the canonical CPython RGB<->YIQ value table (black/blue/green/cyan/red/purple/yellow/white/grey) matches in both directions"""
import colorsys

# YIQ matrix constants are rounded to 4 decimals in the oracle table, so the
# tolerance is intentionally looser than the float-noise EPS used elsewhere.
EPS = 1e-4
# (rgb, yiq) — straight from CPython's test_colorsys.test_yiq_values.
table = [
    ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),             # black
    ((0.0, 0.0, 1.0), (0.11, -0.3217, 0.3121)),     # blue
    ((0.0, 1.0, 0.0), (0.59, -0.2773, -0.5251)),    # green
    ((0.0, 1.0, 1.0), (0.7, -0.599, -0.213)),       # cyan
    ((1.0, 0.0, 0.0), (0.3, 0.599, 0.213)),         # red
    ((1.0, 0.0, 1.0), (0.41, 0.2773, 0.5251)),      # purple
    ((1.0, 1.0, 0.0), (0.89, 0.3217, -0.3121)),     # yellow
    ((1.0, 1.0, 1.0), (1.0, 0.0, 0.0)),             # white
    ((0.5, 0.5, 0.5), (0.5, 0.0, 0.0)),             # grey
]
for rgb, yiq in table:
    got = colorsys.rgb_to_yiq(*rgb)
    for a, b in zip(got, yiq):
        assert abs(a - b) < EPS, ("rgb->yiq", rgb, got)
    back = colorsys.yiq_to_rgb(*yiq)
    for a, b in zip(back, rgb):
        assert abs(a - b) < EPS, ("yiq->rgb", yiq, back)

print("yiq_value_table OK")
