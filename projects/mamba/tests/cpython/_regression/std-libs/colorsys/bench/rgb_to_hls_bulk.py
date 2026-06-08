"""Scalar bulk RGB→HLS conversion (Task #44, Wave-3 ship #1).

Predicted regime: balanced compute-leaning per scout doc. ACTUAL measured
result on 2026-05-15: internal ~0.005x (~150x SLOWER under mamba). Root
cause is NOT the colorsys shim — every per-iter call returns a new
`MbObject::new_tuple(vec![...])`, which goes through `gc::gc_track`
allocation tracking. Contrast cmath.sqrt returning `MbObject::new_complex(r, i)`
(no gc_track call) which PASSES 3.07x internal on the same shape.

This is a runtime-level allocation cost on every tuple-return stdlib fn,
not a colorsys-specific defect. Filed as runtime carve-out; ship colorsys
with Gate 1 (behavior) PASS + Gate 3 (surface 6/6 = 100%) PASS, and Gate 2
expected FAIL until tuple-alloc fast path lands. Same class as #2096
bytes-materialization carve-out — surface ship now, perf follows runtime
infrastructure work.

Hoist convention (#2097): `rgb = colorsys.rgb_to_hls` BEFORE the hot
loop so each iter is a direct func call instead of an attribute lookup.

# tier: compute
"""

import colorsys

# Hoist module-level attribute outside the loop (#2097).
rgb = colorsys.rgb_to_hls

ITERS = 100_000

acc_h = 0.0
acc_l = 0.0
acc_s = 0.0
for i in range(ITERS):
    # Vary all three channels deterministically so the branch tree
    # exercises every leaf of rgb_to_hls (chromatic + achromatic).
    r = 0.10 + (i & 7) * 0.10
    g = 0.50
    b = 0.85 - (i & 3) * 0.05
    h, l, s = rgb(r, g, b)
    acc_h += h
    acc_l += l
    acc_s += s
print("colorsys_rgb_to_hls:", int((acc_h + acc_l + acc_s) * 1000))
