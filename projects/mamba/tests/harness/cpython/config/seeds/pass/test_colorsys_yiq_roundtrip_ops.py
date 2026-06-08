# Operational AssertionPass seed for `colorsys.rgb_to_yiq` /
# `colorsys.yiq_to_rgb` and round-trip identity for arbitrary RGB
# triples through the HLS and HSV color spaces. Surface:
# `rgb_to_yiq(0.5, 0.5, 0.5)` produces Y≈0.5 with chroma components
# near zero (gray maps to a single luma point). `yiq_to_rgb(0.5, 0,
# 0)` returns (0.5, 0.5, 0.5) — gray reconstructs from luma alone.
# A non-primary RGB triple (0.3, 0.6, 0.9) survives `rgb_to_hls` ->
# `hls_to_rgb` and `rgb_to_hsv` -> `hsv_to_rgb` round trips to
# within float-precision tolerance.
import colorsys
import math
_ledger: list[int] = []

# rgb_to_yiq on neutral gray
y, i, q = colorsys.rgb_to_yiq(0.5, 0.5, 0.5)
assert math.isclose(y, 0.5, abs_tol=1e-9); _ledger.append(1)
assert abs(i) < 1e-9; _ledger.append(1)
assert abs(q) < 1e-9; _ledger.append(1)

# yiq_to_rgb on luma-only
r, g, b = colorsys.yiq_to_rgb(0.5, 0.0, 0.0)
assert math.isclose(r, 0.5, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(g, 0.5, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(b, 0.5, abs_tol=1e-9); _ledger.append(1)

# Arbitrary RGB triple round-trips through HLS
r0, g0, b0 = 0.3, 0.6, 0.9
h, l, s = colorsys.rgb_to_hls(r0, g0, b0)
r1, g1, b1 = colorsys.hls_to_rgb(h, l, s)
assert math.isclose(r1, r0, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(g1, g0, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(b1, b0, abs_tol=1e-9); _ledger.append(1)

# Same triple round-trips through HSV
hv, sv, vv = colorsys.rgb_to_hsv(r0, g0, b0)
r2, g2, b2 = colorsys.hsv_to_rgb(hv, sv, vv)
assert math.isclose(r2, r0, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(g2, g0, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(b2, b0, abs_tol=1e-9); _ledger.append(1)

# YIQ luma component identity at primary colors
yR, _, _ = colorsys.rgb_to_yiq(1.0, 0.0, 0.0)
assert 0.0 < yR < 1.0; _ledger.append(1)
yG, _, _ = colorsys.rgb_to_yiq(0.0, 1.0, 0.0)
assert 0.0 < yG < 1.0; _ledger.append(1)
yB, _, _ = colorsys.rgb_to_yiq(0.0, 0.0, 1.0)
assert 0.0 < yB < 1.0; _ledger.append(1)

# Black -> all-zero YIQ
yK, iK, qK = colorsys.rgb_to_yiq(0.0, 0.0, 0.0)
assert math.isclose(yK, 0.0, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(iK, 0.0, abs_tol=1e-9); _ledger.append(1)
assert math.isclose(qK, 0.0, abs_tol=1e-9); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_colorsys_yiq_roundtrip_ops {sum(_ledger)} asserts")
