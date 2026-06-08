# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "real_world"
# case = "palette_conversion_pipeline"
# subject = "colorsys"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys: a UI palette tool round-trips a list of named RGB swatches through HSV, HLS, and YIQ, asserting each recovers the original and aggregating a deterministic checksum over the converted channels"""
import colorsys

# A small design-system palette, normalized to 0..1 channels.
PALETTE = [
    ("slate", (0.28, 0.34, 0.41)),
    ("amber", (1.00, 0.75, 0.00)),
    ("teal", (0.00, 0.50, 0.50)),
    ("rose", (0.86, 0.31, 0.47)),
    ("ink", (0.05, 0.05, 0.08)),
    ("paper", (0.98, 0.97, 0.95)),
]

EPS = 1e-7
checksum = 0.0
for name, (r, g, b) in PALETTE:
    # HSV round-trip (e.g. building a brightness ramp in a color picker).
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    hr, hg, hb = colorsys.hsv_to_rgb(h, s, v)
    assert abs(hr - r) < EPS and abs(hg - g) < EPS and abs(hb - b) < EPS, ("hsv", name)

    # HLS round-trip (e.g. computing lighter/darker shades).
    lh, ll, ls = colorsys.rgb_to_hls(r, g, b)
    lr, lg, lb = colorsys.hls_to_rgb(lh, ll, ls)
    assert abs(lr - r) < EPS and abs(lg - g) < EPS and abs(lb - b) < EPS, ("hls", name)

    # YIQ round-trip (e.g. deriving a grayscale/legacy-broadcast preview).
    y, i, q = colorsys.rgb_to_yiq(r, g, b)
    yr, yg, yb = colorsys.yiq_to_rgb(y, i, q)
    assert abs(yr - r) < EPS and abs(yg - g) < EPS and abs(yb - b) < EPS, ("yiq", name)

    # Aggregate a deterministic checksum over the derived luminance + hue + value.
    checksum += y + h + v

# Integer-scale the checksum so stdout parity does not depend on lsb noise.
print("palette_conversion_pipeline", int(round(checksum * 1000)))
