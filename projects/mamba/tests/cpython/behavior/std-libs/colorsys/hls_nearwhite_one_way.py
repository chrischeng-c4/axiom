# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_nearwhite_one_way"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hls: gh-106498 near-white inputs convert forward to a stable HLS even though the inverse is not exact: rgb_to_hls(0.9999999999999999,1,1)==(0.5,1.0,1.0) and hls_to_rgb of that is (1.0,1.0,1.0)"""
import colorsys

EPS = 1e-9
# These do not round-trip in reverse (gh-106498); only the forward and the
# canonical inverse are stable.
cases = [
    ((0.9999999999999999, 1, 1), (0.5, 1.0, 1.0)),
    ((1, 0.9999999999999999, 0.9999999999999999), (0.0, 1.0, 1.0)),
]
for rgb, hls in cases:
    fwd = colorsys.rgb_to_hls(*rgb)
    for got, want in zip(fwd, hls):
        assert abs(got - want) < EPS, ("forward", rgb, fwd)
    back = colorsys.hls_to_rgb(*hls)
    for got in back:
        assert abs(got - 1.0) < EPS, ("inverse", hls, back)

print("hls_nearwhite_one_way OK")
