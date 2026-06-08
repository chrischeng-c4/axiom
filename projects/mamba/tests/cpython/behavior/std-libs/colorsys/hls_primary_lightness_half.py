# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_primary_lightness_half"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hls: each fully-saturated primary color (red/green/blue) has HLS lightness exactly 0.5"""
import colorsys

EPS = 1e-9
for r, g, b in [(1, 0, 0), (0, 1, 0), (0, 0, 1)]:
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    assert abs(l - 0.5) < EPS, ("primary lightness", (r, g, b), l)

print("hls_primary_lightness_half OK")
