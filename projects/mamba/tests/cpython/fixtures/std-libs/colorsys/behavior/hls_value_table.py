# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_value_table"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hls: the canonical CPython RGB<->HLS value table (black/blue/green/cyan/red/purple/yellow/white/grey) matches in both directions"""
import colorsys

EPS = 1e-7
# (rgb, hls) — straight from CPython's test_colorsys.test_hls_values.
table = [
    ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),         # black
    ((0.0, 0.0, 1.0), (4. / 6., 0.5, 1.0)),     # blue
    ((0.0, 1.0, 0.0), (2. / 6., 0.5, 1.0)),     # green
    ((0.0, 1.0, 1.0), (3. / 6., 0.5, 1.0)),     # cyan
    ((1.0, 0.0, 0.0), (0.0, 0.5, 1.0)),         # red
    ((1.0, 0.0, 1.0), (5. / 6., 0.5, 1.0)),     # purple
    ((1.0, 1.0, 0.0), (1. / 6., 0.5, 1.0)),     # yellow
    ((1.0, 1.0, 1.0), (0.0, 1.0, 0.0)),         # white
    ((0.5, 0.5, 0.5), (0.0, 0.5, 0.0)),         # grey
]
for rgb, hls in table:
    got = colorsys.rgb_to_hls(*rgb)
    for a, b in zip(got, hls):
        assert abs(a - b) < EPS, ("rgb->hls", rgb, got)
    back = colorsys.hls_to_rgb(*hls)
    for a, b in zip(back, rgb):
        assert abs(a - b) < EPS, ("hls->rgb", hls, back)

print("hls_value_table OK")
