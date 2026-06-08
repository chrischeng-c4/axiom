# Operational AssertionPass seed for `colorsys` color-space helpers.
# Surface: rgb_to_hls / hls_to_rgb and rgb_to_hsv / hsv_to_rgb on
# canonical primaries (red/white) — round-trip identity for hls and
# hsv conversions.
# Companion to stub/test_colorsys.py — vendored unittest seed.
import colorsys
import math
_ledger: list[int] = []
# Red — (1, 0, 0) — HLS: hue=0, lightness=0.5, saturation=1
h, l, s = colorsys.rgb_to_hls(1.0, 0.0, 0.0)
assert math.isclose(h, 0.0); _ledger.append(1)
assert math.isclose(l, 0.5); _ledger.append(1)
assert math.isclose(s, 1.0); _ledger.append(1)
# Reverse: HLS(0, 0.5, 1.0) → RGB red
r, g, b = colorsys.hls_to_rgb(0.0, 0.5, 1.0)
assert math.isclose(r, 1.0); _ledger.append(1)
assert math.isclose(g, 0.0); _ledger.append(1)
assert math.isclose(b, 0.0); _ledger.append(1)
# White — (1, 1, 1) — HSV: hue=0, saturation=0, value=1
h2, s2, v2 = colorsys.rgb_to_hsv(1.0, 1.0, 1.0)
assert math.isclose(h2, 0.0); _ledger.append(1)
assert math.isclose(s2, 0.0); _ledger.append(1)
assert math.isclose(v2, 1.0); _ledger.append(1)
# Reverse: HSV(0, 0, 1) → RGB white
r2, g2, b2 = colorsys.hsv_to_rgb(0.0, 0.0, 1.0)
assert math.isclose(r2, 1.0); _ledger.append(1)
assert math.isclose(g2, 1.0); _ledger.append(1)
assert math.isclose(b2, 1.0); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_colorsys_ops {sum(_ledger)} asserts")
