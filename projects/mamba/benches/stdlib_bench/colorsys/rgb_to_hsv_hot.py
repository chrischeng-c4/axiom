"""colorsys.rgb_to_hsv — colour-space convert perf bench.

End-user scenario: `rgb_to_hsv(r, g, b)` inside a tight loop, the
canonical colour transform that backs every theme-derive / palette
generator / image processor hue rotate. CPython routes through
colorsys (pure Python branching on min/max); mamba's colorsys ran
11.5× faster than CPython post-#2100 GC bound fix per
[[project-mamba-2100-gc-bound-resolution]], so this bench documents
the win for stdlib_bench.

Bounded context (DDD): stdlib_bench/colorsys.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `rgb_to_hsv` to a local before the hot loop.
"""

import colorsys
import sys
import time

_rgb_to_hsv = colorsys.rgb_to_hsv

N = 1000
# Spread r/g/b across [0,1) so the min/max branches diversify.
colors = [((i % 256) / 256.0, ((i * 7) % 256) / 256.0, ((i * 13) % 256) / 256.0) for i in range(N)]
ITERS = 1000

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for r, g, b in colors:
        h, s, v = _rgb_to_hsv(r, g, b)
        acc = acc + h + s + v
_t1 = time.perf_counter()

print("rgb_to_hsv_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# All three outputs are in [0,1] so 1 iter contributes 0..3 per color × N.
import math  # noqa: E402
upper = ITERS * N * 3.01
lower = 0.0
assert lower <= acc <= upper, f"acc out of band: {acc}"
# Recompute as float reference and compare.
ref = 0.0
for r, g, b in colors:
    h, s, v = _rgb_to_hsv(r, g, b)
    ref = ref + h + s + v
ref = ref * ITERS
assert math.isclose(acc, ref, rel_tol=1e-6, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {ref}"
)
