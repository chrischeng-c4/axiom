"""Hot-loop bench for stdlib collections: Counter update.

Domain: stdlib
Feature: collections
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop updating a Counter with int values —
monomorphic int regime, measures Counter increment throughput.
"""
# tier: compute

from collections import Counter

ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

_cnt = Counter()
acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _cnt[v] += 1
    acc ^= _cnt[v] & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"counter_update: {ITERS}")
