"""Hot-loop bench for stdlib copy: deepcopy of small dict.

Domain: stdlib
Feature: copy
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop deepcopying a small flat dict —
monomorphic dict regime, measures copy.deepcopy throughput.
"""
# tier: compute

import copy

ITERS = 100_000  # deepcopy is heavier, use fewer iters

_template = {"id": 0, "name": "item", "values": [1, 2, 3], "active": True}
_inputs = [{"id": i, "name": f"item{i}", "values": [i, i+1, i+2], "active": True}
           for i in range(100)]
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    _c = copy.deepcopy(_inputs[i % _n])
    acc ^= _c["id"] & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"deepcopy_dict: {ITERS}")
