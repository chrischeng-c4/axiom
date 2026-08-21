# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "yiq_roundtrip"
# subject = "colorsys.rgb_to_yiq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: rgb_to_yiq then yiq_to_rgb recovers the original RGB across a representative color table"""
import colorsys

# YIQ involves a coarser matrix inverse, so allow a slightly wider tolerance.
EPS = 1e-7
colors = [
    (1.0, 0.0, 0.0),  # red
    (0.0, 1.0, 0.0),  # green
    (0.0, 0.0, 1.0),  # blue
    (0.5, 0.5, 0.5),  # gray
    (0.2, 0.4, 0.8),  # arbitrary
]
for r, g, b in colors:
    y, i, q = colorsys.rgb_to_yiq(r, g, b)
    r2, g2, b2 = colorsys.yiq_to_rgb(y, i, q)
    assert abs(r2 - r) < EPS, ("r", r, r2)
    assert abs(g2 - g) < EPS, ("g", g, g2)
    assert abs(b2 - b) < EPS, ("b", b, b2)

print("yiq_roundtrip OK")
